use pff_sys::libpff_item_t;

use crate::{
    error::Error,
    item::{Item, ItemExt, PffItem},
    recordset::RecordSet,
};

pub struct Recipients {
    recipients: PffItem,
}

impl Recipients {
    pub fn new(recipients: *mut libpff_item_t) -> Self {
        Recipients {
            recipients: PffItem::new(recipients),
        }
    }

    pub fn rs_count(&self) -> Result<i32, Error> {
        self.recipients.record_sets_count()
    }

    pub fn rs(&self) -> Result<Vec<RecordSet>, Error> {
        self.recipients
            .record_sets()?
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect()
    }
}
