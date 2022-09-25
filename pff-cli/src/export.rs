use std::{env, path::PathBuf};
use std::{fs, str};

use anyhow::Result;
use pff::{item::ItemExt, message::Message as PffMessage, FileOpenFlags, Pff};

use crate::index::to_message;

pub(crate) async fn run(
    pff_file: PathBuf,
    save_attachments: bool,
    attachment_save_to: Option<PathBuf>,
    id: String,
) -> Result<()> {
    // parse the id path to the message
    let id_path = id
        .split('_')
        .map(str::parse::<u32>)
        .collect::<Vec<Result<_, _>>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    // open pst/ost file
    let pff = Pff::new()?;
    let pff = pff.open(
        pff_file.as_path().to_str().expect("Path must be valid"),
        FileOpenFlags::READ,
    )?;

    // navigate the tree to get to the message
    let mut item_ref = pff.root_folder()?;
    let mut index = 0;
    let mut message_id = 0;
    while let (Some(item), true) = (item_ref.as_ref(), index < id_path.len()) {
        message_id = id_path[index];
        item_ref = item.sub_item_by_id(message_id)?;
        index += 1;
    }

    if let Some(item) = item_ref {
        let message: PffMessage = item.into();

        if save_attachments && message.has_attachments()? {
            save_all_attachments(&message, attachment_save_to)?;
        }

        let message = to_message(message_id.to_string(), true, message)?;

        // print JSON representation of the message
        println!("{}", serde_json::to_string(&message)?);
    } else {
        eprintln!("Message was not found in the file.");
    }

    Ok(())
}

fn save_all_attachments(message: &PffMessage, save_to: Option<PathBuf>) -> Result<()> {
    let attachments = message.attachments()?;
    let save_to = save_to.unwrap_or(env::current_dir()?);

    for (index, attachment) in attachments.enumerate() {
        let attachment = attachment?;
        let data = attachment.as_buffer()?;
        let name = attachment
            .display_name()?
            .unwrap_or_else(|| format!("attachment_{}", index + 1));
        let save_path = save_to.as_path().join(name);
        fs::write(save_path, &data)?;
    }

    Ok(())
}
