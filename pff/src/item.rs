use std::convert::TryFrom;
use std::ptr;

use num_enum::TryFromPrimitive;
use pff_sys::{
    libpff_error_t, libpff_item_free, libpff_item_get_identifier,
    libpff_item_get_number_of_sub_items, libpff_item_get_sub_item, libpff_item_get_type,
    libpff_item_t,
};

use crate::error::Error;

#[derive(Debug)]
pub struct Item {
    item: *mut libpff_item_t,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            item: ptr::null_mut(),
        }
    }
}

impl Drop for Item {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.item, ptr::null_mut()) };
    }
}

impl Item {
    pub fn new(item: *mut libpff_item_t) -> Self {
        Item { item }
    }

    pub fn id(&self) -> Result<u32, Error> {
        let mut id: u32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_identifier(self.item, &mut id, &mut error) };
        match res {
            1 => Ok(id),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn type_(&self) -> Result<ItemType, Error> {
        let mut item_type: u8 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_type(self.item, &mut item_type, &mut error) };
        match res {
            1 => Ok(ItemType::try_from(item_type).map_err(|_| Error::BadItemType(item_type))?),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn sub_items(&self) -> Result<SubItems<'_>, Error> {
        SubItems::new(self)
    }

    fn sub_items_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_number_of_sub_items(self.item, &mut count, &mut error) };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }
}

pub struct SubItems<'a> {
    item: &'a Item,
    count: i32,
    index: i32,
}

impl<'a> SubItems<'a> {
    pub fn new(item: &'a Item) -> Result<Self, Error> {
        Ok(SubItems {
            item,
            count: item.sub_items_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for SubItems<'a> {
    type Item = Result<Item, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut sub_item: *mut libpff_item_t = ptr::null_mut();
            let res = unsafe {
                libpff_item_get_sub_item(self.item.item, self.index, &mut sub_item, &mut error)
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(Item::new(sub_item)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ItemType {
    Undefined,
    Activity,
    Appointment,
    Attachment,
    Attachments,
    Common,
    Configuration,
    ConflictMessage,
    Contact,
    DistributionList,
    Document,
    Email,
    EmailSmime,
    Fax,
    Folder,
    Meeting,
    Mms,
    Note,
    PostingNote,
    Recipients,
    RssFeed,
    Sharing,
    Sms,
    SubAssociatedContents,
    SubFolders,
    SubMessages,
    Task,
    TaskRequest,
    Voicemail,
    Unknown,
}

#[cfg(test)]
mod tests {
    use crate::{FileOpenFlags, Pff};

    const TEST_PST_FILE: &str = "/media/avranju/data11/rajave-backup/Outlook/avranju@gmail.com.ost";

    #[test]
    fn item_id() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let item = pff.root_item().unwrap().unwrap();
        assert!(item.id().is_ok());
    }

    #[test]
    fn item_type() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let item = pff.root_folder().unwrap().unwrap();
        assert!(item.type_().is_ok());
    }

    #[test]
    fn sub_items_count() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let item = pff.root_folder().unwrap().unwrap();
        assert!(item.sub_items_count().is_ok());
    }

    #[test]
    fn sub_items() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let item = pff.root_folder().unwrap().unwrap();

        for i in item.sub_items().unwrap() {
            println!("{:?}", i.unwrap().type_());
        }
    }
}
