use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Result;
use chrono::NaiveDateTime;
use itertools::Itertools;
use meilisearch_sdk::{client::Client, indexes::Index};
use pff::{
    folder::Folder,
    item::{Item, ItemExt, ItemType},
    message::Message as PffMessage,
    message::MessageBodyType,
    recipients::Recipient,
    FileOpenFlags, Pff,
};
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::progress::{IndexStatus, ProgressTracker};

pub(crate) struct IndexParams {
    pub(crate) pff_file: PathBuf,
    pub(crate) server: String,
    pub(crate) api_key: Option<String>,
    pub(crate) index_name: String,
    pub(crate) progress_file: PathBuf,
    pub(crate) include_body: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Agent {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}

impl Agent {
    fn new(name: Option<String>, email: Option<String>) -> Self {
        Self { name, email }
    }
}

impl From<Recipient> for Agent {
    fn from(recipient: Recipient) -> Self {
        Self::new(recipient.display_name, recipient.email_address)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Body {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) value: String,
}

impl From<(MessageBodyType, String)> for Body {
    fn from((type_, value): (MessageBodyType, String)) -> Self {
        Self {
            type_: type_.to_string(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Message {
    pub(crate) id: String,
    pub(crate) subject: String,
    pub(crate) sender: Agent,
    pub(crate) recipients: Vec<Agent>,
    pub(crate) body: Option<Body>,
    pub(crate) send_time: Option<NaiveDateTime>,
    pub(crate) delivery_time: Option<NaiveDateTime>,
    pub(crate) has_attachments: bool,
    pub(crate) attachments: Option<Vec<String>>,
}

async fn index_messages(
    args: IndexParams,
    mut tracker: ProgressTracker,
    mut rx: mpsc::Receiver<(String, Option<Message>)>,
) -> Result<()> {
    let client = Client::new(&args.server, args.api_key)?;
    let index = client.index(&args.index_name);

    // index messages in batches of 100
    const BATCH_SIZE: usize = 100;
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut index_count = 0usize;

    while let Some((id, message)) = rx.recv().await {
        match message {
            Some(message) => {
                if batch.len() < BATCH_SIZE {
                    batch.push(message);
                } else {
                    index_count += post_to_server(&index, &mut batch, &mut tracker).await?;
                    print!("Indexed {index_count} messages\r");
                    io::stdout().flush()?;
                }
            }
            None => {
                tracker.add_message(id, IndexStatus::Failed);
            }
        }
    }

    // if there are any messages left in the batch, post them
    if !batch.is_empty() {
        index_count += post_to_server(&index, &mut batch, &mut tracker).await?;
        print!("Indexed {index_count} messages\r");
        io::stdout().flush()?;
    }

    Ok(())
}

async fn post_to_server(
    index: &Index,
    batch: &mut Vec<Message>,
    tracker: &mut ProgressTracker,
) -> Result<usize> {
    index.add_documents(&*batch, Some("id")).await?;

    let added = batch.len();

    for message in batch.drain(..) {
        tracker.add_message(message.id, IndexStatus::Indexed);
    }

    Ok(added)
}

fn message_task(
    pff_file: PathBuf,
    include_body: bool,
    tracker: ProgressTracker,
    tx: mpsc::Sender<(String, Option<Message>)>,
) -> Result<()> {
    // open pst/ost file
    let pff = Pff::new()?;
    let pff = pff.open(
        pff_file.as_path().to_str().expect("Path must be valid"),
        FileOpenFlags::READ,
    )?;

    if let Some(root_folder) = pff.root_folder()? {
        let mut id_path = vec![];
        enum_items(root_folder, include_body, &mut id_path, tracker, &tx)?;
    }

    Ok(())
}

fn enum_items<T>(
    root: T,
    include_body: bool,
    id_path: &mut Vec<u32>,
    tracker: ProgressTracker,
    tx: &mpsc::Sender<(String, Option<Message>)>,
) -> Result<()>
where
    T: Item,
{
    if root.type_()? == ItemType::Folder {
        let folder = root.into_folder()?;

        enum_messages(&folder, include_body, id_path, tracker.clone(), tx)?;

        for item in folder.sub_folders()? {
            let item = item?;
            id_path.push(item.id()?);
            enum_items(item, include_body, id_path, tracker.clone(), tx)?;
            id_path.pop();
        }
    }

    Ok(())
}

fn enum_messages(
    folder: &Folder,
    include_body: bool,
    id_path: &[u32],
    tracker: ProgressTracker,
    tx: &mpsc::Sender<(String, Option<Message>)>,
) -> Result<()> {
    for message in folder.messages()? {
        let message = message?;
        let id = format!(
            "{}_{}",
            id_path.iter().map(|id| id.to_string()).join("_"),
            message.id()?
        );

        // skip messages that are already indexed/faulted
        if !tracker.contains_message(&id) {
            tx.blocking_send((id.clone(), to_message(id, include_body, message).ok()))?;
        }
    }

    Ok(())
}

pub(crate) fn to_message(id: String, include_body: bool, message: PffMessage) -> Result<Message> {
    let subject = message.subject()?.unwrap_or_else(|| "--".to_string());
    let sender = Agent::new(message.sender_name()?, message.sender_email_address()?);
    let recipients = message
        .recipients()?
        .and_then(|recs| recs.list().ok())
        .map(|recs| recs.into_iter().map(Agent::from).collect())
        .unwrap_or_default();
    let body = if include_body {
        message.body()?.map(Body::from)
    } else {
        None
    };
    let send_time = message.client_submit_time()?;
    let delivery_time = message.delivery_time()?;
    let has_attachments = message.has_attachments()?;
    let mut attachments = None;
    if has_attachments {
        attachments = Some(
            message
                .attachments()?
                .enumerate()
                .map(|(index, att)| {
                    att.and_then(|att| {
                        att.display_name()
                            .map(|dn| dn.unwrap_or_else(|| format!("attachment_{}", index + 1)))
                    })
                })
                .collect_vec()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
        );
    }

    Ok(Message {
        id,
        subject,
        sender,
        recipients,
        body,
        send_time,
        delivery_time,
        has_attachments,
        attachments,
    })
}

async fn flatten<T>(handle: JoinHandle<Result<T, anyhow::Error>>) -> Result<T, anyhow::Error> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(err.into()),
    }
}

pub(crate) async fn run(args: IndexParams) -> Result<()> {
    let (tx, rx) = mpsc::channel(1024);
    let pff_file = args.pff_file.clone();
    let progress_file = args.progress_file.clone();
    let tracker = ProgressTracker::from_file(&progress_file)?;

    let tracker2 = tracker.clone();
    let h1 = tokio::task::spawn_blocking(move || {
        message_task(pff_file, args.include_body, tracker2, tx)
    });

    let tracker3 = tracker.clone();
    let h2 = tokio::spawn(index_messages(args, tracker3, rx));

    let (_, _) = tokio::try_join!(flatten(h1), flatten(h2))?;
    tracker.to_file(&progress_file)?;

    println!("\nDone.");

    Ok(())
}
