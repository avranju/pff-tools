use std::ptr;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use pff_sys::{
    libpff_attachment_data_read_buffer, libpff_attachment_get_data_size,
    libpff_attachment_get_type, libpff_error_t, libpff_item_free, libpff_item_t,
};

use crate::{error::Error, item::Item};

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum AttachmentType {
    Undefined = 0,
    Data = 'd' as i32,
    Item = 'i' as i32,
    Reference = 'r' as i32,
}

#[derive(Debug)]
pub struct Attachment {
    item: *mut libpff_item_t,
}

impl Default for Attachment {
    fn default() -> Self {
        Attachment {
            item: ptr::null_mut(),
        }
    }
}

impl Drop for Attachment {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.item, ptr::null_mut()) };
    }
}

impl Item for Attachment {
    fn new(item: *mut libpff_item_t) -> Self {
        Attachment { item }
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

impl Attachment {
    pub fn type_(&self) -> Result<AttachmentType, Error> {
        let mut attachment_type: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res =
            unsafe { libpff_attachment_get_type(self.item(), &mut attachment_type, &mut error) };
        match res {
            1 => Ok(AttachmentType::try_from(attachment_type)
                .map_err(|_| Error::BadAttachmentType(attachment_type))?),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn data_size(&self) -> Result<u64, Error> {
        let mut data_size: u64 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res =
            unsafe { libpff_attachment_get_data_size(self.item(), &mut data_size, &mut error) };
        match res {
            1 => Ok(data_size),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn as_buffer(&self) -> Result<Vec<u8>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let data_size = self.data_size()?;
        let mut buf = Vec::<u8>::with_capacity(data_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            let res = libpff_attachment_data_read_buffer(
                self.item(),
                buf_ptr,
                data_size as usize,
                &mut error,
            );
            if res != -1 {
                buf.set_len(res as usize);
            }
            res
        };

        match res {
            -1 => Err(Error::pff_error(error)),
            _ => Ok(buf),
        }
    }
}
