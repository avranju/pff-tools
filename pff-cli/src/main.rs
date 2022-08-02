use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pff::{
    item::{Item, ItemType},
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
        enum_items(&root_folder, 0)?;
    }

    Ok(())
}

fn enum_items(root: &Item, indent: usize) -> Result<()> {
    let item_type = root.type_()?;
    let item_type_str = format!("{:?}", item_type);
    let name = root
        .display_name()
        .unwrap_or_else(|_| Some("*".to_string()))
        .unwrap_or_else(|| "*".to_string());
    println!(
        "{:>ind$} - {name}",
        item_type_str,
        ind = item_type_str.len() + indent,
    );

    if item_type == ItemType::Folder {
        for item in root.sub_items()? {
            let item = item?;
            enum_items(&item, indent + 2)?;
        }
    }

    Ok(())
}
