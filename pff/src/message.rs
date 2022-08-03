use std::{ffi::CString, ptr};

use pff_sys::{
    libpff_error_t, libpff_item_free, libpff_item_t, libpff_message_get_entry_value_utf8_string,
    libpff_message_get_entry_value_utf8_string_size,
};

use crate::{error::Error, item::LibPffEntryType, item_ext::Item};

#[derive(Debug)]
pub struct Message {
    item: *mut libpff_item_t,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            item: ptr::null_mut(),
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.item, ptr::null_mut()) };
    }
}

impl Item for Message {
    fn new(item: *mut libpff_item_t) -> Self {
        Message { item }
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

macro_rules! impl_message_method {
    ($method:ident, $entry_type:ident) => {
        pub fn $method(&self) -> Result<Option<String>, Error> {
            match self.get_entry_string_size(LibPffEntryType::$entry_type)? {
                None => Ok(None),
                Some(entry_size) => self.get_entry_string(LibPffEntryType::$entry_type, entry_size),
            }
        }
    };
}

impl Message {
    impl_message_method!(message_class, MessageClass);
    impl_message_method!(subject, MessageSubject);
    impl_message_method!(conversation_topic, MessageConversationTopic);
    impl_message_method!(sender_name, MessageSenderName);
    impl_message_method!(sender_email_address, MessageSenderEmailAddress);
    impl_message_method!(sent_representing_name, MessageSentRepresentingName);
    impl_message_method!(
        sent_representing_email_address,
        MessageSentRepresentingEmailAddress
    );
    impl_message_method!(received_by_name, MessageReceivedByName);
    impl_message_method!(received_by_email_address, MessageReceivedByEmailAddress);
    impl_message_method!(transport_headers, MessageTransportHeaders);

    fn get_entry_string_size(&self, entry_type: LibPffEntryType) -> Result<Option<u64>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut entry_size: u64 = 0;
        let res = unsafe {
            libpff_message_get_entry_value_utf8_string_size(
                self.item(),
                entry_type.into(),
                &mut entry_size,
                &mut error,
            )
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(entry_size)),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn get_entry_string(
        &self,
        entry_type: LibPffEntryType,
        entry_size: u64,
    ) -> Result<Option<String>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut buf = Vec::<u8>::with_capacity(entry_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            let res = libpff_message_get_entry_value_utf8_string(
                self.item(),
                entry_type.into(),
                buf_ptr,
                entry_size,
                &mut error,
            );
            buf.set_len(entry_size as usize);
            res
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(CString::from_vec_with_nul(buf)?.into_string()?)),
            _ => Err(Error::pff_error(error)),
        }
    }
}
