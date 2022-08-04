use std::{ffi::CString, ptr};

use pff_sys::{
    libpff_error_t, libpff_folder_get_number_of_sub_folders,
    libpff_folder_get_number_of_sub_messages, libpff_folder_get_sub_folder,
    libpff_folder_get_sub_message, libpff_folder_get_utf8_name, libpff_folder_get_utf8_name_size,
    libpff_item_free, libpff_item_t,
};

use crate::{
    error::Error,
    item::{Item, ItemExt, PffItem},
    message::Message,
};

#[derive(Debug)]
pub struct Folder {
    item: *mut libpff_item_t,
}

impl Default for Folder {
    fn default() -> Self {
        Folder {
            item: ptr::null_mut(),
        }
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.item, ptr::null_mut()) };
    }
}

impl Item for Folder {
    fn new(item: *mut libpff_item_t) -> Self {
        Folder { item }
    }

    fn item(&self) -> *mut libpff_item_t {
        self.item
    }

    fn detach(mut self) -> *mut libpff_item_t {
        let item = self.item;
        self.item = ptr::null_mut();
        item
    }
}

impl Folder {
    pub fn name(&self) -> Result<Option<String>, Error> {
        match self.get_name_size()? {
            Some(name_size) if name_size > 0 => self.get_name(name_size),
            _ => Ok(None),
        }
    }

    pub fn get_item_from_id_path(&self, id_path: &[u32]) -> Result<Option<PffItem>, Error> {
        let mut cur = self.sub_item_by_id::<PffItem>(id_path[0])?;
        let mut index = 1;
        while let (Some(si), Some(_)) = (cur.as_ref(), (index < id_path.len()).then_some(())) {
            cur = si.sub_item_by_id(id_path[index])?;
            index += 1;
        }

        Ok(cur)
    }

    pub fn sub_folders_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res =
            unsafe { libpff_folder_get_number_of_sub_folders(self.item(), &mut count, &mut error) };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn sub_folders(&self) -> Result<SubFoldersIterator<'_>, Error> {
        SubFoldersIterator::new(self)
    }

    pub fn messages_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_folder_get_number_of_sub_messages(self.item(), &mut count, &mut error)
        };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn messages(&self) -> Result<SubMessagesIterator<'_>, Error> {
        SubMessagesIterator::new(self)
    }

    fn get_name_size(&self) -> Result<Option<u64>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut name_size: u64 = 0;
        let res =
            unsafe { libpff_folder_get_utf8_name_size(self.item(), &mut name_size, &mut error) };

        match res {
            0 => Ok(None),
            1 => Ok(Some(name_size)),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn get_name(&self, name_size: u64) -> Result<Option<String>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut buf = Vec::<u8>::with_capacity(name_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            let res = libpff_folder_get_utf8_name(self.item(), buf_ptr, name_size, &mut error);
            buf.set_len(name_size as usize);
            res
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(CString::from_vec_with_nul(buf)?.into_string()?)),
            _ => Err(Error::pff_error(error)),
        }
    }
}

pub struct SubFoldersIterator<'a> {
    item: &'a Folder,
    count: i32,
    index: i32,
}

impl<'a> SubFoldersIterator<'a> {
    pub fn new(item: &'a Folder) -> Result<Self, Error> {
        Ok(SubFoldersIterator {
            item,
            count: item.sub_folders_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for SubFoldersIterator<'a> {
    type Item = Result<Folder, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut sub_item: *mut libpff_item_t = ptr::null_mut();
            let res = unsafe {
                libpff_folder_get_sub_folder(
                    self.item.item(),
                    self.index,
                    &mut sub_item,
                    &mut error,
                )
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(Folder::new(sub_item)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}

pub struct SubMessagesIterator<'a> {
    item: &'a Folder,
    count: i32,
    index: i32,
}

impl<'a> SubMessagesIterator<'a> {
    pub fn new(item: &'a Folder) -> Result<Self, Error> {
        Ok(SubMessagesIterator {
            item,
            count: item.messages_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for SubMessagesIterator<'a> {
    type Item = Result<Message, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut sub_item: *mut libpff_item_t = ptr::null_mut();
            let res = unsafe {
                libpff_folder_get_sub_message(
                    self.item.item(),
                    self.index,
                    &mut sub_item,
                    &mut error,
                )
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(Message::new(sub_item)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}
