use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod export;
mod index;
mod progress;

#[derive(Parser, Debug)]
#[clap(version)]
struct Opts {
    #[clap(long, short)]
    /// Path to PST/OST file
    pff_file: PathBuf,

    /// The command to run
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, PartialOrd, Ord, Eq, Debug, PartialEq)]
pub(crate) enum Command {
    /// Export a single message as JSON
    ExportMessage {
        #[clap(long, short)]
        /// The ID of the message to export. The ID must be given as
        /// as a sequence '_' delimited numbers. For example, 8354_8514_8546_7029316.
        /// This ID can be fetched from the Meilisearch server search results.
        /// Note that this message ID path must not include the root folder's ID
        /// which is what you get by default if you indexed your emails using the
        /// `pff-cli index` command.
        id: String,

        /// Should attachments (if any) be saved to the file system
        #[clap(long, short)]
        save_attachments: bool,

        /// If '--save-attachments' is set then a path to a folder where the
        /// attachments are to be saved can be specified here. If this is not
        /// specified then the path defaults to the current folder.
        #[clap(long, short)]
        attachment_save_to: Option<PathBuf>,
    },

    /// Index all emails to a Meilisearch server
    Index {
        #[clap(long, short)]
        /// Search server URL in form "http://ip:port" or "http://hostname:port"
        server: String,

        #[clap(long, short)]
        /// Search server API key (if any)
        api_key: Option<String>,

        #[clap(long, short)]
        /// Index name
        index_name: String,

        #[clap(long, short = 'f', default_value = "progress.csv")]
        /// File to save progress to so we can resume later
        progress_file: PathBuf,

        #[clap(long, short = 'b', action)]
        /// Should the message body be included in the index?
        include_body: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Opts::parse();
    let pff_file = args.pff_file;

    match args.command {
        Command::ExportMessage {
            id,
            save_attachments,
            attachment_save_to,
        } => export::run(pff_file, save_attachments, attachment_save_to, id).await,

        Command::Index {
            server,
            api_key,
            index_name,
            progress_file,
            include_body,
        } => {
            let params = index::IndexParams {
                pff_file,
                server,
                api_key,
                index_name,
                progress_file,
                include_body,
            };
            index::run(params).await
        }
    }
}
