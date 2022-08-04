use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use pff::{
    folder::Folder,
    item::{Item, ItemExt, ItemType},
    message::Message,
    FileOpenFlags, Pff,
};

#[derive(Parser, Debug)]
#[clap(version)]
struct Opt {
    /// Path to PST/OST file
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Opt::parse();
    let pff = Pff::new()?;
    let pff = pff.open(
        args.file.as_path().to_str().expect("Path must be valid"),
        FileOpenFlags::READ,
    )?;
    // if let Some(root_folder) = pff.root_folder()? {
    //     let mut id_path = vec![root_folder.id()?];
    //     _enum_items(root_folder, &mut id_path, 0)?;
    // }

    let id_path = vec![8354, 8514, 32834, 32930, 2105828];
    // let id_path = vec![8354, 8514, 8578, 2114724];
    let folder = pff.root_folder()?.unwrap().into_folder()?;
    if let Some(item) = folder.get_item_from_id_path(&id_path)? {
        let msg: Message = item.into();
        println!(
            "Subject: {}",
            msg.subject()?.as_ref().map(AsRef::as_ref).unwrap_or("-")
        );
        if let Some(recipients) = msg.recipients()? {
            let rs = recipients.list()?;
            println!("To: {}", rs.iter().map(|r| r.to_string()).join(", "));
        }
    }

    Ok(())
}

fn _enum_items<T: Item>(root: T, id_path: &mut Vec<u32>, indent: usize) -> Result<()> {
    let item_type = root.type_()?;
    let item_type_str = format!("{:?}", item_type);
    let name = root
        .display_name()
        .unwrap_or_else(|_| Some("*".to_string()))
        .unwrap_or_else(|| "*".to_string());

    if item_type == ItemType::Folder {
        let folder = root.into_folder()?;

        let entries_count = folder.entries_count()?;
        let msg_count = folder.messages_count()?;

        let path_str = id_path.iter().map(|id| id.to_string()).join("/");
        println!(
            "{:>ind$} - [{path_str}] - {name} - {entries_count} - {msg_count}",
            item_type_str,
            ind = item_type_str.len() + indent,
        );

        _enum_messages(&folder, indent + 2)?;

        for item in folder.sub_folders()? {
            let item = item?;
            id_path.push(item.id()?);
            _enum_items(item, id_path, indent + 2)?;
            id_path.pop();
        }
    }

    Ok(())
}

fn _enum_messages(folder: &Folder, indent: usize) -> Result<()> {
    const TYPE: &str = "Message";
    for message in folder.messages()? {
        let message = message?;
        let subject = message.subject()?.unwrap_or_else(|| "--".to_string());
        let submit_time = message.client_submit_time()?;
        let id = message.id()?;
        let count = if let Some(recipients) = message.recipients()? {
            let rs = recipients.rs()?;
            rs.iter()
                .try_fold(0, |acc, r| r.entries_count().map(|c| acc + c))?
        } else {
            0
        };
        println!(
            "{:>ind$} - [{id}] {submit_time:?} {count} - {subject}",
            TYPE,
            ind = TYPE.len() + indent,
        );
    }

    Ok(())
}
