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

        println!(
            "From: {}",
            msg.sender()?.as_ref().map(AsRef::as_ref).unwrap_or("-")
        );

        if let Some(recipients) = msg.recipients()? {
            println!(
                "To: {}",
                recipients
                    .list()?
                    .iter()
                    .map(ToString::to_string)
                    .join(", ")
            );
        }

        // if let Some(cp) = msg.first_entry_by_type(EntryType::MessageBodyCodepage)? {
        //     println!("type = {:?}", cp.value_type()?);
        //     println!("codepage = {}", cp.as_u32()?);
        // }

        // {
        //     for rec in msg.record_sets()? {
        //         let rec = rec?;
        //         for ent in rec.entries()? {
        //             let ent = ent?;
        //             println!("  {:?} {:?}", ent.type_()?, ent.value_type()?);
        //         }
        //         println!("---");
        //     }
        // }

        if let Some((body_type, body)) = msg.body()? {
            println!("body type = {:?}", body_type);
            println!("body = {}", body);
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
        println!(
            "{:>ind$} - [{id}] {submit_time:?} - {subject}",
            TYPE,
            ind = TYPE.len() + indent,
        );
    }

    Ok(())
}
