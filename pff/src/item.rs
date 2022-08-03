use std::ptr;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use pff_sys::{libpff_item_free, libpff_item_t};

use crate::item_ext::Item;

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
    use crate::{item_ext::ItemExt, FileOpenFlags, Pff};

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
