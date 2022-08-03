use std::ptr;

use pff_sys::{libpff_record_set_free, libpff_record_set_t};

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
}
