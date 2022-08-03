use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pff::{
    folder::Folder,
    item::ItemType,
    item_ext::{Item, ItemExt},
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
    if let Some(root_folder) = pff.root_folder()? {
        enum_items(root_folder, 0)?;
    }

    Ok(())
}

fn enum_items<T: Item>(root: T, indent: usize) -> Result<()> {
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
        println!(
            "{:>ind$} - {name} - {entries_count} - {msg_count}",
            item_type_str,
            ind = item_type_str.len() + indent,
        );

        enum_messages(&folder, indent + 2)?;

        for item in folder.sub_folders()? {
            let item = item?;
            enum_items(item, indent + 2)?;
        }
    }

    Ok(())
}

fn enum_messages(folder: &Folder, indent: usize) -> Result<()> {
    const TYPE: &str = "Message";
    for message in folder.messages()? {
        let message = message?;
        let subject = message.subject()?.unwrap_or_else(|| "--".to_string());
        println!("{:>ind$} - {subject}", TYPE, ind = TYPE.len() + indent,);
    }

    Ok(())
}
