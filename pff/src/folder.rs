use std::{ffi::CString, ptr};

use pff_sys::{
    libpff_error_t, libpff_folder_get_utf8_name, libpff_folder_get_utf8_name_size,
    libpff_item_free, libpff_item_t,
};

use crate::{error::Error, item_ext::Item};

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
