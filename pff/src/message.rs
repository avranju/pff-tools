use std::{fmt::Display, ptr};

use chrono::NaiveDateTime;
use concat_idents::concat_idents;
use pff_sys::{
    libpff_error_t, libpff_item_free, libpff_item_t, libpff_message_get_attachment,
    libpff_message_get_client_submit_time, libpff_message_get_creation_time,
    libpff_message_get_delivery_time, libpff_message_get_entry_value_utf8_string,
    libpff_message_get_entry_value_utf8_string_size, libpff_message_get_html_body,
    libpff_message_get_html_body_size, libpff_message_get_modification_time,
    libpff_message_get_number_of_attachments, libpff_message_get_plain_text_body,
    libpff_message_get_plain_text_body_size, libpff_message_get_recipients,
    libpff_message_get_rtf_body, libpff_message_get_rtf_body_size,
};

use crate::{
    attachment::Attachment,
    encoding,
    error::Error,
    filetime::FileTime,
    item::{EntryType, Item},
    recipients::Recipients,
};

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

macro_rules! prop_string {
    ($method:ident, $entry_type:ident) => {
        pub fn $method(&self) -> Result<Option<String>, Error> {
            match self.get_entry_string_size(EntryType::$entry_type)? {
                Some(entry_size) if entry_size > 0 => {
                    self.get_entry_string(EntryType::$entry_type, entry_size)
                }
                _ => Ok(None),
            }
        }
    };
}

macro_rules! prop_time {
    ($method:ident) => {
        concat_idents!(fn_name = libpff_message_get_, $method {
            pub fn $method(&self) -> Result<Option<NaiveDateTime>, Error> {
                let mut error: *mut libpff_error_t = ptr::null_mut();
                let mut time: u64 = 0;
                let res = unsafe { fn_name(self.item(), &mut time, &mut error) };

                match res {
                    0 => Ok(None),
                    1 => Ok(Some(FileTime(time as i64).into())),
                    _ => Err(Error::pff_error(error)),
                }
            }
        });
    };
}

macro_rules! prop_body {
    ($fn_name:ident, $pff_size_fn_name:ident, $pff_fn_name:ident) => {
        pub fn $fn_name(&self) -> Result<Option<String>, Error> {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut body_size = 0;
            let res = unsafe { $pff_size_fn_name(self.item(), &mut body_size, &mut error) };

            match res {
                0 => Ok(None),
                1 => {
                    let mut buf = Vec::<u8>::with_capacity(body_size as usize);
                    let buf_ptr = buf.as_mut_ptr();

                    let res = unsafe {
                        let res = $pff_fn_name(self.item(), buf_ptr, body_size, &mut error);
                        if res == 1 {
                            buf.set_len(body_size as usize);
                        }
                        res
                    };

                    match res {
                        0 => Ok(None),
                        1 => Ok(Some(encoding::try_get_item_string(
                            self,
                            EntryType::MessageBodyCodepage,
                            buf,
                        )?)),
                        _ => Err(Error::pff_error(error)),
                    }
                }
                _ => Err(Error::pff_error(error)),
            }
        }
    };
}

#[derive(Debug)]
pub enum MessageBodyType {
    PlainText,
    Html,
    Rtf,
}

impl Display for MessageBodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MessageBodyType::PlainText => write!(f, "plain"),
            MessageBodyType::Html => write!(f, "html"),
            MessageBodyType::Rtf => write!(f, "rtf"),
        }
    }
}

impl Message {
    prop_string!(message_class, MessageClass);
    prop_string!(subject, MessageSubject);
    prop_string!(conversation_topic, MessageConversationTopic);
    prop_string!(sender_name, MessageSenderName);
    prop_string!(sender_email_address, MessageSenderEmailAddress);
    prop_string!(sent_representing_name, MessageSentRepresentingName);
    prop_string!(
        sent_representing_email_address,
        MessageSentRepresentingEmailAddress
    );
    prop_string!(received_by_name, MessageReceivedByName);
    prop_string!(received_by_email_address, MessageReceivedByEmailAddress);
    prop_string!(transport_headers, MessageTransportHeaders);

    prop_time!(client_submit_time);
    prop_time!(delivery_time);
    prop_time!(creation_time);
    prop_time!(modification_time);

    prop_body!(
        plain_text_body,
        libpff_message_get_plain_text_body_size,
        libpff_message_get_plain_text_body
    );
    prop_body!(
        rtf_body,
        libpff_message_get_rtf_body_size,
        libpff_message_get_rtf_body
    );
    prop_body!(
        html_body,
        libpff_message_get_html_body_size,
        libpff_message_get_html_body
    );

    pub fn body(&self) -> Result<Option<(MessageBodyType, String)>, Error> {
        // try getting the body in this order: html, plain text, rtf
        match self.html_body()? {
            Some(body) => Ok(Some((MessageBodyType::Html, body))),
            None => match self.plain_text_body()? {
                Some(body) => Ok(Some((MessageBodyType::PlainText, body))),
                None => match self.rtf_body()? {
                    Some(body) => Ok(Some((MessageBodyType::Rtf, body))),
                    None => Ok(None),
                },
            },
        }
    }

    pub fn sender(&self) -> Result<Option<String>, Error> {
        let sender_name = self.sender_name()?;
        let sender_email = self.sender_email_address()?;

        match (sender_name, sender_email) {
            (Some(name), Some(email)) => Ok(Some(format!("{name} <{email}>"))),
            (Some(name), None) => Ok(Some(name)),
            (None, Some(email)) => Ok(Some(email)),
            _ => Ok(None),
        }
    }

    pub fn received_by(&self) -> Result<Option<String>, Error> {
        let received_by_name = self.received_by_name()?;
        let received_by_email_address = self.received_by_email_address()?;

        match (received_by_name, received_by_email_address) {
            (Some(name), Some(email)) => Ok(Some(format!("{name} <{email}>"))),
            (Some(name), None) => Ok(Some(name)),
            (None, Some(email)) => Ok(Some(email)),
            _ => Ok(None),
        }
    }

    pub fn recipients(&self) -> Result<Option<Recipients>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut recipients: *mut libpff_item_t = ptr::null_mut();

        let res =
            unsafe { libpff_message_get_recipients(self.item(), &mut recipients, &mut error) };

        match res {
            0 => Ok(None),
            1 => Ok(Some(Recipients::new(recipients))),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn has_attachments(&self) -> Result<bool, Error> {
        Ok(attachments_count(self)? > 0)
    }

    pub fn attachments(&self) -> Result<AttachmentsIterator<'_>, Error> {
        AttachmentsIterator::new(self)
    }

    fn get_entry_string_size(&self, entry_type: EntryType) -> Result<Option<usize>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut entry_size = 0;
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
        entry_type: EntryType,
        entry_size: usize,
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
            if res == 1 {
                buf.set_len(entry_size as usize);
            }
            res
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(encoding::try_get_item_string(
                self,
                EntryType::MessageCodepage,
                buf,
            )?)),
            _ => Err(Error::pff_error(error)),
        }
    }
}

pub struct AttachmentsIterator<'a> {
    message: &'a Message,
    count: i32,
    index: i32,
}

impl<'a> AttachmentsIterator<'a> {
    pub fn new(message: &'a Message) -> Result<Self, Error> {
        Ok(Self {
            message,
            count: attachments_count(message)?,
            index: 0,
        })
    }
}

impl<'a> Iterator for AttachmentsIterator<'a> {
    type Item = Result<Attachment, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut attachment: *mut libpff_item_t = ptr::null_mut();
            let res = unsafe {
                libpff_message_get_attachment(
                    self.message.item(),
                    self.index,
                    &mut attachment,
                    &mut error,
                )
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(Attachment::new(attachment)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}

fn attachments_count(message: &Message) -> Result<i32, Error> {
    // get number of attachments
    let mut count: i32 = 0;
    let mut error: *mut libpff_error_t = ptr::null_mut();

    let res =
        unsafe { libpff_message_get_number_of_attachments(message.item(), &mut count, &mut error) };
    match res {
        1 => Ok(count),
        _ => Err(Error::pff_error(error)),
    }
}
