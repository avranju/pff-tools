use std::ptr;

use pff_sys::{
    libpff_error_t, libpff_record_entry_t, libpff_record_set_free,
    libpff_record_set_get_entry_by_index, libpff_record_set_get_number_of_entries,
    libpff_record_set_t,
};

use crate::{error::Error, record_entry::RecordEntry};

#[derive(Debug)]
pub struct RecordSet {
    record_set: *mut libpff_record_set_t,
}

impl Default for RecordSet {
    fn default() -> Self {
        RecordSet {
            record_set: ptr::null_mut(),
        }
    }
}

impl Drop for RecordSet {
    fn drop(&mut self) {
        unsafe { libpff_record_set_free(&mut self.record_set, ptr::null_mut()) };
    }
}

impl RecordSet {
    pub fn new(record_set: *mut libpff_record_set_t) -> Self {
        Self { record_set }
    }

    pub fn entries(&self) -> Result<RecordEntryIterator<'_>, Error> {
        RecordEntryIterator::new(self)
    }

    pub fn entries_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_set_get_number_of_entries(self.record_set, &mut count, &mut error)
        };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }
}

pub struct RecordEntryIterator<'a> {
    record_set: &'a RecordSet,
    count: i32,
    index: i32,
}

impl<'a> RecordEntryIterator<'a> {
    pub fn new(record_set: &'a RecordSet) -> Result<Self, Error> {
        Ok(RecordEntryIterator {
            record_set,
            count: record_set.entries_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for RecordEntryIterator<'a> {
    type Item = Result<RecordEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let mut error: *mut libpff_error_t = ptr::null_mut();
            let mut record_entry: *mut libpff_record_entry_t = ptr::null_mut();
            let res = unsafe {
                libpff_record_set_get_entry_by_index(
                    self.record_set.record_set,
                    self.index,
                    &mut record_entry,
                    &mut error,
                )
            };

            match res {
                1 => {
                    self.index += 1;
                    Some(Ok(RecordEntry::new(record_entry)))
                }
                _ => Some(Err(Error::pff_error(error))),
            }
        }
    }
}
