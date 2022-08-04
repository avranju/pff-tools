use std::ptr;

use pff_sys::{
    libpff_error_t, libpff_multi_value_free, libpff_multi_value_get_number_of_values,
    libpff_multi_value_t,
};

use crate::{error::Error, multivalue_entry::MultiValueEntry};

#[derive(Debug)]
pub struct MultiValue {
    multi_value: *mut libpff_multi_value_t,
}

impl Default for MultiValue {
    fn default() -> Self {
        MultiValue {
            multi_value: ptr::null_mut(),
        }
    }
}

impl Drop for MultiValue {
    fn drop(&mut self) {
        unsafe { libpff_multi_value_free(&mut self.multi_value, ptr::null_mut()) };
    }
}

impl MultiValue {
    pub fn new(multi_value: *mut libpff_multi_value_t) -> Self {
        Self { multi_value }
    }

    pub fn as_ptr(&self) -> *mut libpff_multi_value_t {
        self.multi_value
    }

    pub fn entries(&self) -> Result<MultiValueEntryIterator<'_>, Error> {
        MultiValueEntryIterator::new(self)
    }

    pub fn entries_count(&self) -> Result<i32, Error> {
        let mut count: i32 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_multi_value_get_number_of_values(self.multi_value, &mut count, &mut error)
        };
        match res {
            1 => Ok(count),
            _ => Err(Error::pff_error(error)),
        }
    }
}

pub struct MultiValueEntryIterator<'a> {
    multi_value: &'a MultiValue,
    count: i32,
    index: i32,
}

impl<'a> MultiValueEntryIterator<'a> {
    pub fn new(multi_value: &'a MultiValue) -> Result<Self, Error> {
        Ok(MultiValueEntryIterator {
            multi_value,
            count: multi_value.entries_count()?,
            index: 0,
        })
    }
}

impl<'a> Iterator for MultiValueEntryIterator<'a> {
    type Item = Result<MultiValueEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            Some(MultiValueEntry::new(self.multi_value, self.index))
        }
    }
}
