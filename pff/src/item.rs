use std::ptr;
use std::{convert::TryFrom, ffi::CString};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use pff_sys::{
    libpff_error_t, libpff_item_free, libpff_item_get_entry_value_utf8_string,
    libpff_item_get_entry_value_utf8_string_size, libpff_item_get_identifier,
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

    pub fn display_name(&self) -> Result<Option<String>, Error> {
        match self.get_string_size(LibPffEntryType::DisplayName)? {
            None => Ok(None),
            Some(str_size) => self.get_string(LibPffEntryType::DisplayName, str_size),
        }
    }

    fn get_string_size(&self, entry_type: LibPffEntryType) -> Result<Option<u64>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut str_size: u64 = 0;
        let res = unsafe {
            libpff_item_get_entry_value_utf8_string_size(
                self.item,
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

    fn get_string(
        &self,
        entry_type: LibPffEntryType,
        str_size: u64,
    ) -> Result<Option<String>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut buf = Vec::<u8>::with_capacity(str_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            libpff_item_get_entry_value_utf8_string(
                self.item,
                0,
                entry_type.into(),
                buf_ptr,
                str_size,
                0,
                &mut error,
            )
        };

        match res {
            0 => Ok(None),
            1 => Ok(Some(CString::from_vec_with_nul(buf)?.into_string()?)),
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

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum LibPffEntryType {
    MessageImportance = 0x0017,
    MessageClass = 0x001a,
    MessagePriority = 0x0026,
    MessageSensitivity = 0x0036,
    MessageSubject = 0x0037,
    MessageClientSubmitTime = 0x0039,
    MessageSentRepresentingSearchKey = 0x003b,
    MessageReceivedByEntryIdentifier = 0x003f,
    MessageReceivedByName = 0x0040,
    MessageSentRepresentingEntryIdentifier = 0x0041,
    MessageSentRepresentingName = 0x0042,
    MessageReceivedRepresentingEntryIdentifier = 0x0043,
    MessageReceivedRepresentingName = 0x0044,
    MessageReplyRecipientEntries = 0x004f,
    MessageReplyRecipientNames = 0x0050,
    MessageReceivedBySearchKey = 0x0051,
    MessageReceivedRepresentingSearchKey = 0x0052,
    MessageSentRepresentingAddressType = 0x0064,
    MessageSentRepresentingEmailAddress = 0x0065,
    MessageConversationTopic = 0x0070,
    MessageConversationIndex = 0x0071,
    MessageReceivedByAddressType = 0x0075,
    MessageReceivedByEmailAddress = 0x0076,
    MessageReceivedRepresentingAddressType = 0x0077,
    MessageReceivedRepresentingEmailAddress = 0x0078,
    MessageTransportHeaders = 0x007d,
    RecipientType = 0x0c15,
    MessageSenderEntryIdentifier = 0x0c19,
    MessageSenderName = 0x0c1a,
    MessageSenderSearchKey = 0x0c1d,
    MessageSenderAddressType = 0x0c1e,
    MessageSenderEmailAddress = 0x0c1f,
    MessageDisplayTo = 0x0e04,
    MessageDeliveryTime = 0x0e06,
    MessageFlags = 0x0e07,
    MessageSize = 0x0e08,
    MessageStatus = 0x0e17,
    AttachmentSize = 0x0e20,
    MessageInternetArticleNumber = 0x0e23,
    MessagePermission = 0x0e27,
    MessageUrlComputerNameSet = 0x0e62,
    MessageTrustSender = 0x0e79,
    MessageBodyPlainText = 0x1000,
    MessageBodyCompressedRtf = 0x1009,
    MessageBodyHtml = 0x1013,
    EmailEmlFilename = 0x10f3,
    DisplayName = 0x3001,
    AddressType = 0x3002,
    EmailAddress = 0x3003,
    MessageCreationTime = 0x3007,
    MessageModificationTime = 0x3008,
    MessageStoreValidFolderMask = 0x35df,
    FolderType = 0x3601,
    NumberOfContentItems = 0x3602,
    NumberOfUnreadContentItems = 0x3603,
    HasSubFolders = 0x360a,
    ContainerClass = 0x3613,
    NumberOfAssociatedContent = 0x3617,
    AttachmentDataObject = 0x3701,
    AttachmentFilenameShort = 0x3704,
    AttachmentMethod = 0x3705,
    AttachmentFilenameLong = 0x3707,
    AttachmentRenderingPosition = 0x370b,
    ContactCallbackPhoneNumber = 0x3a02,
    ContactGenerationalAbbreviation = 0x3a05,
    ContactGivenName = 0x3a06,
    ContactBusinessPhoneNumber1 = 0x3a08,
    ContactHomePhoneNumber = 0x3a09,
    ContactInitials = 0x3a0a,
    ContactSurname = 0x3a11,
    ContactPostalAddress = 0x3a15,
    ContactCompanyName = 0x3a16,
    ContactJobTitle = 0x3a17,
    ContactDepartmentName = 0x3a18,
    ContactOfficeLocation = 0x3a19,
    ContactPrimaryPhoneNumber = 0x3a1a,
    ContactBusinessPhoneNumber2 = 0x3a1b,
    ContactMobilePhoneNumber = 0x3a1c,
    ContactBusinessFaxNumber = 0x3a24,
    ContactCountry = 0x3a26,
    ContactLocality = 0x3a27,
    ContactTitle = 0x3a45,
    MessageBodyCodepage = 0x3fde,
    MessageCodepage = 0x3ffd,
    RecipientDisplayName = 0x5ff6,
    FolderChildCount = 0x6638,
    SubItemIdentifier = 0x67f2,
    MessageStorePasswordChecksum = 0x67ff,
    AddressFileUnder = 0x8005,
    DistributionListName = 0x8053,
    DistributionListMemberOneOffEntryIdentifiers = 0x8054,
    DistributionListMemberEntryIdentifiers = 0x8055,
    ContactEmailAddress1 = 0x8083,
    ContactEmailAddress2 = 0x8093,
    ContactEmailAddress3 = 0x80a3,
    TaskStatus = 0x8101,
    TaskPercentageComplete = 0x8102,
    TaskStartDate = 0x8104,
    TaskDueDate = 0x8105,
    TaskActualEffort = 0x8110,
    TaskTotalEffort = 0x8111,
    TaskVersion = 0x8112,
    TaskIsComplete = 0x811c,
    TaskIsRecurring = 0x8126,
    AppointmentBusyStatus = 0x8205,
    AppointmentLocation = 0x8208,
    AppointmentStartTime = 0x820d,
    AppointmentEndTime = 0x820e,
    AppointmentDuration = 0x8213,
    AppointmentIsRecurring = 0x8223,
    AppointmentRecurrencePattern = 0x8232,
    AppointmentTimezoneDescription = 0x8234,
    AppointmentFirstEffectiveTime = 0x8235,
    AppointmentLastEffectiveTime = 0x8236,
    MessageReminderTime = 0x8502,
    MessageIsReminder = 0x8503,
    MessageIsPrivate = 0x8506,
    MessageReminderSignalTime = 0x8550,
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

    const TEST_PST_FILE: &str = "/Users/avranju/Downloads/outlook/rajave@microsoft.com.nst";

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
            println!("{:?}", i.unwrap().type_().unwrap());
        }
    }
}
