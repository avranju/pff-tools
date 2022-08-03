use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pff::{
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
    let entries_count = root.entries_count()?;
    let recordsets_count = root.record_sets_count()?;
    println!(
        "{:>ind$} - {name} - {entries_count} - {recordsets_count}",
        item_type_str,
        ind = item_type_str.len() + indent,
    );

    if item_type == ItemType::Folder {
        let folder = root.into_folder()?;
        for item in folder.sub_items()? {
            let item = item?;
            enum_items(item, indent + 2)?;
        }
    }

    Ok(())
}
