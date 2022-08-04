use std::{ffi::CString, ptr};

use chrono::NaiveDateTime;
use concat_idents::concat_idents;
use pff_sys::{
    libpff_error_t, libpff_multi_value_free, libpff_multi_value_get_number_of_values,
    libpff_multi_value_get_value, libpff_multi_value_get_value_32bit,
    libpff_multi_value_get_value_64bit, libpff_multi_value_get_value_binary_data,
    libpff_multi_value_get_value_binary_data_size, libpff_multi_value_get_value_filetime,
    libpff_multi_value_get_value_guid, libpff_multi_value_get_value_utf8_string,
    libpff_multi_value_get_value_utf8_string_size, libpff_multi_value_t,
};
use uuid::Uuid;

use crate::{error::Error, filetime::FileTime, item::ValueType};

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

pub struct MultiValueEntry<'a> {
    multi_value: &'a MultiValue,
    index: i32,
    value_type: ValueType,
    value_size: u64,
}

macro_rules! data_get {
    ($as_type:ty, $fn_name:ident, $pff_type:ident) => {
        concat_idents!(pff_fn_name = libpff_multi_value_get_value, $pff_type {
            pub fn $fn_name(&self) -> Result<$as_type, Error> {
                let mut error: *mut libpff_error_t = ptr::null_mut();
                let mut val: $as_type = Default::default();
                let res = unsafe { pff_fn_name(self.multi_value.as_ptr(), self.index, &mut val, &mut error) };

                match res {
                    1 => Ok(val),
                    _ => Err(Error::pff_error(error)),
                }
            }
        });
    };
}

impl<'a> MultiValueEntry<'a> {
    pub fn new(multi_value: &'a MultiValue, index: i32) -> Result<Self, Error> {
        let mut value_type: u32 = 0;
        let mut value_data: *mut u8 = ptr::null_mut();
        let mut value_size: u64 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_multi_value_get_value(
                multi_value.as_ptr(),
                index,
                &mut value_type,
                &mut value_data,
                &mut value_size,
                &mut error,
            )
        };
        match res {
            1 => Ok(Self {
                multi_value,
                index,
                value_type: ValueType::try_from(value_type)
                    .map_err(|_| Error::BadValueType(value_type))?,
                value_size,
            }),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    pub fn value_size(&self) -> u64 {
        self.value_size
    }

    fn string_size(&self) -> Result<u64, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut str_size: u64 = 0;
        let res = unsafe {
            libpff_multi_value_get_value_utf8_string_size(
                self.multi_value.as_ptr(),
                self.index,
                &mut str_size,
                &mut error,
            )
        };

        match res {
            1 => Ok(str_size),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn as_string(&self) -> Result<String, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let str_size = self.string_size()?;
        let mut buf = Vec::<u8>::with_capacity(str_size as usize);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe {
            let res = libpff_multi_value_get_value_utf8_string(
                self.multi_value.as_ptr(),
                self.index,
                buf_ptr,
                str_size,
                &mut error,
            );
            buf.set_len(str_size as usize);
            res
        };

        match res {
            1 => Ok(CString::from_vec_with_nul(buf)?.into_string()?),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn data_size(&self) -> Result<u64, Error> {
        let mut data_size: u64 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_multi_value_get_value_binary_data_size(
                self.multi_value.as_ptr(),
                self.index,
                &mut data_size,
                &mut error,
            )
        };
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
            let res = libpff_multi_value_get_value_binary_data(
                self.multi_value.as_ptr(),
                self.index,
                buf_ptr,
                data_size,
                &mut error,
            );
            buf.set_len(data_size as usize);
            res
        };

        match res {
            1 => Ok(buf),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn as_guid(&self) -> Result<Uuid, Error> {
        let mut buf: [u8; 16] = [0; 16];
        let buf_ptr = buf.as_mut_ptr();
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_multi_value_get_value_guid(
                self.multi_value.as_ptr(),
                self.index,
                buf_ptr,
                buf.len() as u64,
                &mut error,
            )
        };
        match res {
            1 => Ok(Uuid::from_slice(&buf)?),
            _ => Err(Error::pff_error(error)),
        }
    }

    pub fn as_filetime(&self) -> Result<NaiveDateTime, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut val: u64 = 0;
        let res = unsafe {
            libpff_multi_value_get_value_filetime(
                self.multi_value.as_ptr(),
                self.index,
                &mut val,
                &mut error,
            )
        };

        match res {
            1 => Ok(FileTime(val as i64).into()),
            _ => Err(Error::pff_error(error)),
        }
    }

    data_get!(u32, as_u32, _32bit);
    data_get!(u64, as_u64, _64bit);
}
