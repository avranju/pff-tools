use std::{ffi::CString, ptr};

use pff_sys::{
    libpff_error_t, libpff_item_get_entry_value_utf8_string,
    libpff_item_get_entry_value_utf8_string_size, libpff_item_get_identifier,
    libpff_item_get_number_of_entries, libpff_item_get_number_of_record_sets,
    libpff_item_get_number_of_sub_items, libpff_item_get_record_set_by_index,
    libpff_item_get_sub_item, libpff_item_get_sub_item_by_identifier, libpff_item_get_type,
    libpff_item_t, libpff_record_set_t,
};

use crate::{
    error::Error,
    folder::Folder,
    item::{EntryType, ItemType},
    record_set::RecordSet,
};

pub trait Item {
    fn new(item: *mut libpff_item_t) -> Self;
    fn item(&self) -> *mut libpff_item_t;
    fn detach(self) -> *mut libpff_item_t;
}

pub trait ItemExt: Item + Sized {
    fn id(&self) -> Result<u32, Error> {
        let mut id: u32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_identifier(self.item(), &mut id, &mut error) };
        match res {
            1 => Ok(id),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn type_(&self) -> Result<ItemType, Error> {
        let mut item_type: u8 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_type(self.item(), &mut item_type, &mut error) };
        match res {
            1 => Ok(ItemType::try_from(item_type).map_err(|_| Error::BadItemType(item_type))?),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn sub_items(&self) -> Result<SubItemsIterator<'_, Self>, Error> {
        SubItemsIterator::new(self)
    }

    fn sub_item_by_id(&self, id: u32) -> Result<Option<Self>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut sub_item: *mut libpff_item_t = ptr::null_mut();

        let res = unsafe {
            libpff_item_get_sub_item_by_identifier(self.item(), id, &mut sub_item, &mut error)
        };
        match res {
            1 => Ok(Some(Self::new(sub_item))),
            0 => Ok(None),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn entries_count(&self) -> Result<u32, Error> {
        let mut count: u32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_item_get_number_of_entries(self.item(), &mut count, &mut error) };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn record_sets_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res =
            unsafe { libpff_item_get_number_of_record_sets(self.item(), &mut count, &mut error) };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn sub_items_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res =
            unsafe { libpff_item_get_number_of_sub_items(self.item(), &mut count, &mut error) };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn display_name(&self) -> Result<Option<String>, Error> {
        match self.get_string_size(EntryType::DisplayName)? {
            None => Ok(None),
            Some(str_size) => self.get_string(EntryType::DisplayName, str_size),
        }
    }

    fn get_string_size(&self, entry_type: EntryType) -> Result<Option<u64>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut str_size: u64 = 0;
        let res = unsafe {
            libpff_item_get_entry_value_utf8_string_size(
                self.item(),
                0,
                entry_type.into(),
                &mut str_size,
                0,
                &mut error,
            )
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(str_size)),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn get_string(&self, entry_type: EntryType, str_size: u64) -> Result<Option<String>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut buf = Vec::<u8>::with_capacity(str_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            let res = libpff_item_get_entry_value_utf8_string(
                self.item(),
                0,
                entry_type.into(),
                buf_ptr,
                str_size,
                0,
                &mut error,
            );
            buf.set_len(str_size as usize);
            res
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(CString::from_vec_with_nul(buf)?.into_string()?)),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn into_folder(self) -> Result<Folder, Error> {
        match self.type_()? {
            ItemType::Folder => Ok(Folder::new(self.detach())),
            _ => Err(Error::NotAFolder),
        }
    }
}

/// Blanket impl of `ItemExt` for all `T`s that implement `Item`.
impl<T: Item> ItemExt for T {}

pub struct SubItemsIterator<'a, T> {
    item: &'a T,
    count: i32,
    index: i32,
}

impl<'a, T: Item + ItemExt> SubItemsIterator<'a, T> {
    pub fn new(item: &'a T) -> Result<Self, Error> {
        Ok(SubItemsIterator {
            item,
            count: item.sub_items_count()?,
            index: 0,
        })
    }
}

impl<'a, T: Item> Iterator for SubItemsIterator<'a, T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut sub_item: *mut libpff_item_t = ptr::null_mut();
            let res = unsafe {
                libpff_item_get_sub_item(self.item.item(), self.index, &mut sub_item, &mut error)
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(T::new(sub_item)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}

pub struct RecordSetIterator<'a, T> {
    item: &'a T,
    count: i32,
    index: i32,
}

impl<'a, T: Item + ItemExt> RecordSetIterator<'a, T> {
    pub fn new(item: &'a T) -> Result<Self, Error> {
        Ok(RecordSetIterator {
            item,
            count: item.record_sets_count()?,
            index: 0,
        })
    }
}

impl<'a, T: Item> Iterator for RecordSetIterator<'a, T> {
    type Item = Result<RecordSet, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut record_set: *mut libpff_record_set_t = ptr::null_mut();
            let res = unsafe {
                libpff_item_get_record_set_by_index(
                    self.item.item(),
                    self.index,
                    &mut record_set,
                    &mut error,
                )
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(RecordSet::new(record_set)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}
