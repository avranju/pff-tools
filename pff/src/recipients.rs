use std::{collections::BTreeMap, fmt::Display, ptr};

use itertools::Itertools;
use pff_sys::{libpff_item_free, libpff_item_t};

use crate::{
    error::Error,
    item::{EntryType, Item, ItemExt},
    recordset::RecordSet,
};

#[derive(Debug, Default)]
pub struct Recipient {
    pub email_address: Option<String>,
    pub display_name: Option<String>,
    pub address_type: Option<String>,
}

impl<T> From<T> for Recipient
where
    T: Iterator<Item = (EntryType, String)>,
{
    fn from(it: T) -> Self {
        let map = it.collect::<BTreeMap<_, _>>();

        let address_type = map.get(&EntryType::AddressType);
        let email_address = address_type.and_then(|at| {
            // For Microsoft Exchange PSTs, the address type is "EX"
            // and the email address is in EmailAddress2
            if at.as_str() == "EX" {
                map.get(&EntryType::EmailAddress2)
            } else {
                map.get(&EntryType::EmailAddress)
            }
        });

        Recipient {
            email_address: email_address.map(Clone::clone),
            display_name: map.get(&EntryType::DisplayName).map(Clone::clone),
            address_type: address_type.map(Clone::clone),
        }
    }
}

pub struct Recipients {
    recipients: *mut libpff_item_t,
}

impl Default for Recipients {
    fn default() -> Self {
        Recipients {
            recipients: ptr::null_mut(),
        }
    }
}

impl Drop for Recipients {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.recipients, ptr::null_mut()) };
    }
}

impl Item for Recipients {
    fn new(recipients: *mut libpff_item_t) -> Self {
        Recipients { recipients }
    }

    fn item(&self) -> *mut libpff_item_t {
        self.recipients
    }

    fn detach(mut self) -> *mut libpff_item_t {
        let recipients = self.recipients;
        self.recipients = ptr::null_mut();
        recipients
    }
}

impl Recipients {
    pub fn rs_count(&self) -> Result<i32, Error> {
        self.record_sets_count()
    }

    pub fn rs(&self) -> Result<Vec<RecordSet>, Error> {
        self.record_sets()?
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect()
    }

    pub fn list(&self) -> Result<Vec<Recipient>, Error> {
        self.record_sets()?
            .map_ok(record_set_to_recipient)
            .flatten_ok()
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }
}

impl Display for Recipient {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.display_name.as_ref(), self.email_address.as_ref()) {
            (Some(dn), Some(ea)) => write!(f, "{} <{}>", dn, ea),
            (Some(dn), None) => write!(f, "{}", dn),
            (None, Some(ea)) => write!(f, "{}", ea),
            (None, None) => write!(f, ""),
        }
    }
}

fn record_set_to_recipient(rs: RecordSet) -> Result<Recipient, Error> {
    let mut email_address1 = None;
    let mut email_address2 = None;
    let mut display_name = None;
    let mut address_type = None;

    for entry in rs.entries()? {
        let entry = entry?;

        match entry.type_() {
            Ok(EntryType::EmailAddress) => {
                email_address1 = Some(entry.as_string()?);
            }
            Ok(EntryType::EmailAddress2) => {
                email_address2 = Some(entry.as_string()?);
            }
            Ok(EntryType::DisplayName) => {
                display_name = Some(entry.as_string()?);
            }
            Ok(EntryType::AddressType) => {
                address_type = Some(entry.as_string()?);
            }
            _ => {}
        }
    }

    // For Microsoft Exchange PSTs, the address type is "EX"
    // and the email address is in EmailAddress2
    let mut email_address = email_address1;
    if let Some(at) = address_type.as_ref() {
        if at.as_str() == "EX" {
            email_address = email_address2;
        }
    }

    Ok(Recipient {
        email_address,
        display_name,
        address_type,
    })
}
