use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use csv::{ReaderBuilder, Writer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum IndexStatus {
    /// Message has been indexed
    Indexed,

    /// Message failed to load and won't be indexed
    Failed,
}

#[derive(Clone)]
pub(crate) struct ProgressTracker {
    pub messages: Arc<Mutex<BTreeMap<String, IndexStatus>>>,
}

impl ProgressTracker {
    pub(crate) fn from_file(path: &Path) -> Result<Self> {
        let mut messages = BTreeMap::new();

        if path.exists() {
            let mut rdr = ReaderBuilder::new().has_headers(false).from_path(path)?;
            for result in rdr.deserialize() {
                let (id, status) = result?;
                messages.insert(id, status);
            }
        }

        Ok(Self {
            messages: Arc::new(Mutex::new(messages)),
        })
    }

    pub(crate) fn add_message(&mut self, id: String, status: IndexStatus) {
        self.messages
            .lock()
            .unwrap()
            .entry(id)
            .and_modify(|e| *e = status)
            .or_insert(status);
    }

    pub(crate) fn contains_message(&self, id: &String) -> bool {
        self.messages.lock().unwrap().contains_key(id)
    }

    pub(crate) fn to_file(&self, path: &Path) -> Result<()> {
        let mut wtr = Writer::from_path(path)?;
        for (id, status) in self.messages.lock().unwrap().iter() {
            wtr.serialize((id, status))?;
        }
        Ok(())
    }
}
