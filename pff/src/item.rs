use std::ptr;

use bitflags::bitflags;
use itertools::Itertools;
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use pff_sys::{
    libpff_error_t, libpff_item_free, libpff_item_get_entry_value_utf8_string,
    libpff_item_get_entry_value_utf8_string_size, libpff_item_get_identifier,
    libpff_item_get_number_of_entries, libpff_item_get_number_of_record_sets,
    libpff_item_get_number_of_sub_items, libpff_item_get_record_set_by_index,
    libpff_item_get_sub_item, libpff_item_get_sub_item_by_identifier, libpff_item_get_type,
    libpff_item_t, libpff_record_set_t,
};

use crate::{
    encoding,
    error::Error,
    folder::Folder,
    recordset::{RecordEntry, RecordSet},
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

    fn sub_item_by_id<T: Item>(&self, id: u32) -> Result<Option<T>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut sub_item: *mut libpff_item_t = ptr::null_mut();

        let res = unsafe {
            libpff_item_get_sub_item_by_identifier(self.item(), id, &mut sub_item, &mut error)
        };
        match res {
            1 => Ok(Some(T::new(sub_item))),
            0 => Ok(None),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn first_entry_by_type(&self, entry_type: EntryType) -> Result<Option<RecordEntry>, Error> {
        self.record_sets()?
            .map_ok(|rs| rs.entry_by_type(entry_type))
            .flatten_ok()
            .find(|e| e.as_ref().map(|e| e.is_some()).ok().is_some())
            .unwrap_or(Ok(None))
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

    fn record_sets(&self) -> Result<RecordSetIterator<'_, Self>, Error> {
        RecordSetIterator::new(self)
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

    fn get_string_size(&self, entry_type: EntryType) -> Result<Option<usize>, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut str_size = 0;
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

    fn get_string(&self, entry_type: EntryType, str_size: usize) -> Result<Option<String>, Error> {
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
            if res == 1 {
                buf.set_len(str_size as usize);
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

#[derive(Debug)]
pub struct PffItem {
    item: *mut libpff_item_t,
}

impl Default for PffItem {
    fn default() -> Self {
        PffItem {
            item: ptr::null_mut(),
        }
    }
}

impl Drop for PffItem {
    fn drop(&mut self) {
        unsafe { libpff_item_free(&mut self.item, ptr::null_mut()) };
    }
}

impl Item for PffItem {
    fn new(item: *mut libpff_item_t) -> Self {
        PffItem { item }
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

impl PffItem {
    pub fn into<T: Item>(self) -> T {
        T::new(self.detach())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ValueType {
    Unspecified = 0x0000,
    Null = 0x0001,
    Integer16BitSigned = 0x0002,
    Integer32BitSigned = 0x0003,
    Float32Bit = 0x0004,
    Double64Bit = 0x0005,
    Currency = 0x0006,
    Floatingtime = 0x0007,
    Error = 0x000a,
    Boolean = 0x000b,
    Object = 0x000d,
    Integer64BitSigned = 0x0014,
    StringAscii = 0x001e,
    StringUnicode = 0x001f,
    Filetime = 0x0040,
    Guid = 0x0048,
    ServerIdentifier = 0x00fb,
    Restriction = 0x00fd,
    RuleAction = 0x00fe,
    BinaryData = 0x0102,
    MultiValueInteger16BitSigned = 0x1002,
    MultiValueInteger32BitSigned = 0x1003,
    MultiValueFloat32Bit = 0x1004,
    MultiValueDouble64Bit = 0x1005,
    MultiValueCurrency = 0x1006,
    MultiValueFloatingtime = 0x1007,
    MultiValueInteger64BitSigned = 0x1014,
    MultiValueStringAscii = 0x101e,
    MultiValueStringUnicode = 0x101f,
    MultiValueFiletime = 0x1040,
    MultiValueGuid = 0x1048,
    MultiValueBinaryData = 0x1102,
}

bitflags! {
    pub struct ValueFlags: u8 {
        const MATCH_ANY_VALUE_TYPE = 0x01;
        const IGNORE_NAME_TO_ID_MAP = 0x02;
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum EntryType {
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
    EmailAddress2 = 0x39FE,
    Alias = 0x39FF,
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
    #[num_enum(default)]
    Unknown,
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
    use crate::{item::ItemExt, FileOpenFlags, Pff};

    const TEST_PST_FILE: &str = "../data/sample.ost";

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
