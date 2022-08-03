use std::ptr;

use pff_sys::{
    libpff_error_t, libpff_record_entry_free, libpff_record_entry_get_entry_type,
    libpff_record_entry_get_value_type, libpff_record_entry_t,
};

use crate::{
    error::Error,
    item::{EntryType, ValueType},
};

#[derive(Debug)]
pub struct RecordEntry {
    record_entry: *mut libpff_record_entry_t,
}

impl Default for RecordEntry {
    fn default() -> Self {
        RecordEntry {
            record_entry: ptr::null_mut(),
        }
    }
}

impl Drop for RecordEntry {
    fn drop(&mut self) {
        unsafe { libpff_record_entry_free(&mut self.record_entry, ptr::null_mut()) };
    }
}

impl RecordEntry {
    pub fn new(record_entry: *mut libpff_record_entry_t) -> Self {
        Self { record_entry }
    }

    pub fn type_(&self) -> Result<EntryType, Error> {
        let mut entry_type: u32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_entry_get_entry_type(self.record_entry, &mut entry_type, &mut error)
        };
        match res {
            1 => Ok(EntryType::try_from(entry_type).map_err(|_| Error::BadEntryType(entry_type))?),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn value_type(&self) -> Result<ValueType, Error> {
        let mut value_type: u32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_entry_get_value_type(self.record_entry, &mut value_type, &mut error)
        };
        match res {
            1 => Ok(ValueType::try_from(value_type).map_err(|_| Error::BadValueType(value_type))?),
            _ => Err(Error::pff_error(error)),
        }
    }
}
