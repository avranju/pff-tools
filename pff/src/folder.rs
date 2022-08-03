use std::{ffi::CString, ptr};

use pff_sys::{
    libpff_error_t, libpff_folder_get_number_of_sub_folders,
    libpff_folder_get_number_of_sub_messages, libpff_folder_get_sub_folder,
    libpff_folder_get_sub_message, libpff_folder_get_utf8_name, libpff_folder_get_utf8_name_size,
    libpff_item_free, libpff_item_t,
};

use crate::{error::Error, item_ext::Item, message::Message};

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

    pub fn sub_folders(&self) -> Result<SubFolders<'_>, Error> {
        SubFolders::new(self)
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

    pub fn messages(&self) -> Result<SubMessages<'_>, Error> {
        SubMessages::new(self)
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

pub struct SubFolders<'a> {
    item: &'a Folder,
    count: i32,
    index: i32,
}

impl<'a> SubFolders<'a> {
    pub fn new(item: &'a Folder) -> Result<Self, Error> {
        Ok(SubFolders {
            item,
            count: item.sub_folders_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for SubFolders<'a> {
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

pub struct SubMessages<'a> {
    item: &'a Folder,
    count: i32,
    index: i32,
}

impl<'a> SubMessages<'a> {
    pub fn new(item: &'a Folder) -> Result<Self, Error> {
        Ok(SubMessages {
            item,
            count: item.messages_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for SubMessages<'a> {
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
