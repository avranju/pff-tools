use anyhow::Result;
use meilisearch_sdk::client::Client;

use crate::index::Message;

pub(crate) struct SearchParams {
    pub(crate) server: String,
    pub(crate) api_key: Option<String>,
    pub(crate) index_name: String,
    pub(crate) query: String,
    pub(crate) offset: Option<usize>,
    pub(crate) fetch_all: bool,
    pub(crate) has_attachments: bool,
}

pub(crate) async fn run(params: SearchParams) -> Result<()> {
    let client = Client::new(
        &params.server,
        params.api_key.as_ref().map(AsRef::as_ref).unwrap_or(""),
    );
    let index = client.index(&params.index_name);
    let mut search = index.search();
    let mut offset = params.offset.unwrap_or_default();
    let query = search.with_query(&params.query).with_offset(offset);

    let query = if params.has_attachments {
        query.with_filter("has_attachments = true")
    } else {
        query
    };

    print!("[");

    let mut results = query.execute::<Message>().await?;
    print!(
        "{}",
        serde_json::to_string(&results.hits.iter().map(|h| &h.result).collect::<Vec<_>>())?
    );

    if params.fetch_all {
        while !results.hits.is_empty() {
            offset += results.hits.len();
            query.with_offset(offset);
            results = query.execute().await?;
            print!(
                ",{}",
                serde_json::to_string(&results.hits.iter().map(|h| &h.result).collect::<Vec<_>>())?
            );
        }
    }

    println!("]");

    Ok(())
}
