use chrono::NaiveDateTime;
use meilisearch_sdk::client::Client;
use pff::message::MessageBodyType;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Agent {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Body {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) value: String,
}

impl From<(MessageBodyType, String)> for Body {
    fn from((type_, value): (MessageBodyType, String)) -> Self {
        Self {
            type_: type_.to_string(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Message {
    pub(crate) id: String,
    pub(crate) subject: String,
    pub(crate) sender: Agent,
    pub(crate) recipients: Vec<Agent>,
    pub(crate) body: Option<Body>,
    pub(crate) send_time: Option<NaiveDateTime>,
    pub(crate) delivery_time: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SearchResult {
    pub(crate) messages: Vec<Message>,
    pub(crate) total_matches: usize,
    pub(crate) offset: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchClient {
    client: Client,
    index_name: String,
}

impl SearchClient {
    pub fn new(endpoint: String, api_key: String, index_name: String) -> Self {
        Self {
            client: Client::new(endpoint, api_key),
            index_name,
        }
    }

    pub async fn search(
        &self,
        query_str: String,
        offset: Option<usize>,
    ) -> Result<SearchResult, Error> {
        let index = self.client.index(&self.index_name);
        let mut query = index.search();
        query.query = Some(&query_str);
        query.offset = offset;
        let results = query.execute::<Message>().await?;

        Ok(SearchResult {
            messages: results.hits.into_iter().map(|v| v.result).collect(),
            offset: results.offset,
            total_matches: results.nb_hits,
        })
    }
}
