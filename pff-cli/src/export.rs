use std::path::PathBuf;
use std::str;

use anyhow::Result;
use pff::{item::ItemExt, message::Message as PffMessage, FileOpenFlags, Pff};

use crate::index::to_message;

pub(crate) async fn run(pff_file: PathBuf, id: String) -> Result<()> {
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
        let message = to_message(message_id.to_string(), true, message)?;

        // print JSON representation of the message
        println!("{}", serde_json::to_string(&message)?);
    } else {
        eprintln!("Message was not found in the file.");
    }

    Ok(())
}
