use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::trace;
use pff::{item::ItemExt, message::Message as PffMessage, FileOpenFlags, Pff, PffOpen};
use tokio::{
    sync::{mpsc, watch},
    time::timeout,
};

use crate::{error::Error, search::Body};

const BACKLOG: usize = 1024;

struct State {
    pff_file: PathBuf,
    command_tx: mpsc::Sender<String>,
    body_tx: mpsc::Sender<(String, Result<Body, Error>)>,
    body_cache: BTreeMap<String, Result<Body, Error>>,
    status_rx: watch::Receiver<()>,
}

#[derive(Clone)]
pub(crate) struct PffManager {
    state: Arc<Mutex<State>>,
}

impl PffManager {
    pub(crate) fn new(pff_file: PathBuf) -> Self {
        let (command_tx, command_rx) = mpsc::channel(BACKLOG);
        let (body_tx, mut body_rx) = mpsc::channel(BACKLOG);
        let (status_tx, status_rx) = watch::channel(());

        let manager = Self {
            state: Arc::new(Mutex::new(State {
                pff_file,
                command_tx,
                body_tx,
                body_cache: BTreeMap::new(),
                status_rx,
            })),
        };

        // kick off the pff manager loop
        tokio::task::spawn_blocking({
            let manager = manager.clone();
            move || pff_loop(manager, command_rx)
        });

        // kick off loop for caching message bodies
        tokio::task::spawn({
            let manager = manager.clone();
            async move {
                while let Some((id, body)) = body_rx.recv().await {
                    manager.state.lock().unwrap().body_cache.insert(id, body);

                    // we don't care if this fails since the thread waiting
                    // on the other end might have timed out
                    let _ = status_tx.send(());
                }
            }
        });

        manager
    }

    pub(crate) async fn get_body(
        &self,
        id: String,
        timeout_duration: Duration,
    ) -> Result<Body, Error> {
        // send the request to the pff manager loop
        let command_tx = self.state.lock().unwrap().command_tx.clone();
        command_tx
            .send(id.clone())
            .await
            .map_err(|_| Error::PffChannelClosed)?;

        // wait for the response
        let mut status_rx = self.state.lock().unwrap().status_rx.clone();
        match timeout(timeout_duration, status_rx.changed()).await {
            Ok(Ok(_)) => Ok(self
                .state
                .lock()
                .unwrap()
                .body_cache
                .remove(&id)
                .ok_or(Error::BodyNotFound)??),
            Ok(Err(_)) => Err(Error::BodyCacheLoopClosed),
            Err(_) => Err(Error::BodyTimeout),
        }
    }
}

fn pff_loop(manager: PffManager, mut command_rx: mpsc::Receiver<String>) -> Result<(), Error> {
    let pff_file = manager.state.lock().unwrap().pff_file.clone();

    trace!("Loading PFF file: {:?}", pff_file);
    let pff = Pff::new()?;
    let pff = pff.open(
        pff_file
            .as_path()
            .to_str()
            .expect("Path to PFF file is invalid."),
        FileOpenFlags::READ,
    )?;
    trace!("PFF opened.");

    let message_tx = manager.state.lock().unwrap().body_tx.clone();
    while let Some(message_id) = command_rx.blocking_recv() {
        let body = locate_message(&pff, &message_id);
        message_tx
            .blocking_send((message_id, body))
            .map_err(|_| Error::BodyChannelClosed)?;
    }

    Ok(())
}

fn locate_message(pff: &PffOpen, id: &str) -> Result<Body, Error> {
    // parse the id path to the message
    let id_path = id
        .split('_')
        .map(str::parse::<u32>)
        .collect::<Vec<Result<_, _>>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    // navigate the tree to get to the message
    let mut item_ref = pff.root_folder()?;
    let mut index = 0;
    while let (Some(item), true) = (item_ref.as_ref(), index < id_path.len()) {
        item_ref = item.sub_item_by_id(id_path[index])?;
        index += 1;
    }

    if let Some(item) = item_ref {
        let message: PffMessage = item.into();
        Ok(message.body()?.map(Body::from).ok_or(Error::BodyNotFound)?)
    } else {
        Err(Error::BodyNotFound)
    }
}
