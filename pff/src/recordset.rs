use std::{ffi::CString, ptr};

use chrono::NaiveDateTime;
use concat_idents::concat_idents;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pff_sys::{
    libpff_error_t, libpff_multi_value_t, libpff_record_entry_free, libpff_record_entry_get_data,
    libpff_record_entry_get_data_as_16bit_integer, libpff_record_entry_get_data_as_32bit_integer,
    libpff_record_entry_get_data_as_64bit_integer, libpff_record_entry_get_data_as_boolean,
    libpff_record_entry_get_data_as_filetime, libpff_record_entry_get_data_as_floating_point,
    libpff_record_entry_get_data_as_floatingtime, libpff_record_entry_get_data_as_guid,
    libpff_record_entry_get_data_as_size, libpff_record_entry_get_data_as_utf8_string,
    libpff_record_entry_get_data_as_utf8_string_size, libpff_record_entry_get_data_size,
    libpff_record_entry_get_entry_type, libpff_record_entry_get_multi_value,
    libpff_record_entry_get_value_type, libpff_record_entry_read_buffer,
    libpff_record_entry_seek_offset, libpff_record_entry_t, libpff_record_set_free,
    libpff_record_set_get_entry_by_index, libpff_record_set_get_number_of_entries,
    libpff_record_set_t,
};
use uuid::Uuid;

use crate::{
    error::Error,
    filetime::FileTime,
    item::{EntryType, ValueType},
    multivalue::MultiValue,
};

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

macro_rules! data_get {
    ($as_type:ty, $fn_name:ident, $pff_type:ident) => {
        concat_idents!(pff_fn_name = libpff_record_entry_get_data_, $pff_type {
            pub fn $fn_name(&self) -> Result<$as_type, Error> {
                let mut error: *mut libpff_error_t = ptr::null_mut();
                let mut val: $as_type = Default::default();
                let res = unsafe { pff_fn_name(self.record_entry, &mut val, &mut error) };

                match res {
                    1 => Ok(val),
                    _ => Err(Error::pff_error(error)),
                }
            }
        });
    };
}

macro_rules! data_get_time {
    ($fn_name:ident, $pff_type:ident) => {
        concat_idents!(pff_fn_name = libpff_record_entry_get_data_, $pff_type {
            pub fn $fn_name(&self) -> Result<NaiveDateTime, Error> {
                let mut error: *mut libpff_error_t = ptr::null_mut();
                let mut val: u64 = 0;
                let res = unsafe { pff_fn_name(self.record_entry, &mut val, &mut error) };

                match res {
                    1 => Ok(FileTime(val as i64).into()),
                    _ => Err(Error::pff_error(error)),
                }
            }
        });
    };
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum Seek {
    Set = 0,
    Current = 1,
    End = 2,
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

    pub fn data_size(&self) -> Result<u64, Error> {
        let mut data_size: u64 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_entry_get_data_size(self.record_entry, &mut data_size, &mut error)
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
            let res =
                libpff_record_entry_get_data(self.record_entry, buf_ptr, data_size, &mut error);
            buf.set_len(data_size as usize);
            res
        };

        match res {
            1 => Ok(buf),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn seek_offset(&self, offset: i64, whence: Seek) -> Result<i64, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let res = unsafe {
            libpff_record_entry_seek_offset(self.record_entry, offset, whence.into(), &mut error)
        };

        match res {
            -1 => Err(Error::pff_error(error)),
            _ => Ok(res),
        }
    }

    pub fn as_buffer_from_offset(
        &self,
        offset: i64,
        whence: Seek,
        mut buf: Vec<u8>,
    ) -> Result<i64, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let buf_ptr = buf.as_mut_ptr();

        self.seek_offset(offset, whence)?;

        let res = unsafe {
            libpff_record_entry_read_buffer(
                self.record_entry,
                buf_ptr,
                buf.len() as u64,
                &mut error,
            )
        };

        match res {
            -1 => Err(Error::pff_error(error)),
            _ => Ok(res),
        }
    }

    pub fn as_bool(&self) -> Result<bool, Error> {
        let mut val: u8 = 0;
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_entry_get_data_as_boolean(self.record_entry, &mut val, &mut error)
        };
        match res {
            1 => Ok(val == 1),
            _ => Err(Error::pff_error(error)),
        }
    }

    fn string_size(&self) -> Result<u64, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut str_size: u64 = 0;
        let res = unsafe {
            libpff_record_entry_get_data_as_utf8_string_size(
                self.record_entry,
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
            let res = libpff_record_entry_get_data_as_utf8_string(
                self.record_entry,
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

    pub fn as_guid(&self) -> Result<Uuid, Error> {
        let mut buf: [u8; 16] = [0; 16];
        let buf_ptr = buf.as_mut_ptr();
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe {
            libpff_record_entry_get_data_as_guid(
                self.record_entry,
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

    pub fn as_multi_value(&self) -> Result<MultiValue, Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut multi_value: *mut libpff_multi_value_t = ptr::null_mut();
        let res = unsafe {
            libpff_record_entry_get_multi_value(self.record_entry, &mut multi_value, &mut error)
        };
        match res {
            1 => Ok(MultiValue::new(multi_value)),
            _ => Err(Error::pff_error(error)),
        }
    }

    // TODO: libpff_record_entry_get_name_to_id_map_entry

    data_get!(u16, as_u16, as_16bit_integer);
    data_get!(u32, as_u32, as_32bit_integer);
    data_get!(u64, as_u64, as_64bit_integer);
    data_get_time!(as_filetime, as_filetime);
    data_get_time!(as_floatingtime, as_floatingtime);
    data_get!(u64, as_size, as_size);
    data_get!(f64, as_f64, as_floating_point);
}
