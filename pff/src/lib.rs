use std::{ffi::CString, ptr};

use bitflags::bitflags;
use item::Item;
use pff_sys::{
    libpff_error_t, libpff_file_close, libpff_file_free, libpff_file_get_root_folder,
    libpff_file_get_root_item, libpff_file_get_size, libpff_file_initialize, libpff_file_open,
    libpff_file_t, libpff_item_t, LIBPFF_ACCESS_FLAGS_LIBPFF_ACCESS_FLAG_READ,
    LIBPFF_ACCESS_FLAGS_LIBPFF_ACCESS_FLAG_WRITE,
};

pub mod error;
mod filetime;
pub mod folder;
pub mod item;
pub mod message;
pub mod multivalue;
pub mod multivalue_entry;
pub mod record_entry;
pub mod record_set;

#[derive(Debug)]
pub struct Pff {
    file: *mut libpff_file_t,
}

impl Pff {
    pub fn new() -> Result<Self, error::Error> {
        let mut file: *mut libpff_file_t = ptr::null_mut();
        let mut error: *mut libpff_error_t = ptr::null_mut();

        let res = unsafe { libpff_file_initialize(&mut file, &mut error) };
        match res {
            1 => Ok(Pff { file }),
            _ => Err(error::Error::pff_error(error)),
        }
    }

    pub fn open(mut self, path: &str, open_flags: FileOpenFlags) -> Result<PffOpen, error::Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let path_str = CString::new(path)?;
        let res = unsafe {
            libpff_file_open(
                self.file,
                path_str.as_ptr(),
                open_flags.bits as i32,
                &mut error,
            )
        };
        match res {
            1 => {
                let pff_open = PffOpen { file: self.file };
                self.file = ptr::null_mut();
                Ok(pff_open)
            }
            _ => Err(error::Error::pff_error(error)),
        }
    }
}

impl Drop for Pff {
    fn drop(&mut self) {
        unsafe { libpff_file_free(&mut self.file, ptr::null_mut()) };
    }
}

#[derive(Debug)]
pub struct PffOpen {
    file: *mut libpff_file_t,
}

impl PffOpen {
    pub fn size(&self) -> Result<Option<u64>, error::Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut size: u64 = 0;
        let res = unsafe { libpff_file_get_size(self.file, &mut size, &mut error) };
        match res {
            1 => Ok(Some(size)),
            0 => Ok(None),
            _ => Err(error::Error::pff_error(error)),
        }
    }

    pub fn root_item(&self) -> Result<Option<item::PffItem>, error::Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut item: *mut libpff_item_t = ptr::null_mut();
        let res = unsafe { libpff_file_get_root_item(self.file, &mut item, &mut error) };
        match res {
            1 => Ok(Some(item::PffItem::new(item))),
            0 => Ok(None),
            _ => Err(error::Error::pff_error(error)),
        }
    }

    pub fn root_folder(&self) -> Result<Option<item::PffItem>, error::Error> {
        let mut error: *mut libpff_error_t = ptr::null_mut();
        let mut item: *mut libpff_item_t = ptr::null_mut();
        let res = unsafe { libpff_file_get_root_folder(self.file, &mut item, &mut error) };
        match res {
            1 => Ok(Some(item::PffItem::new(item))),
            0 => Ok(None),
            _ => Err(error::Error::pff_error(error)),
        }
    }
}

impl Drop for PffOpen {
    fn drop(&mut self) {
        unsafe { libpff_file_close(self.file, ptr::null_mut()) };
        unsafe { libpff_file_free(&mut self.file, ptr::null_mut()) };
    }
}

bitflags! {
    pub struct FileOpenFlags: u32 {
        const READ = LIBPFF_ACCESS_FLAGS_LIBPFF_ACCESS_FLAG_READ;
        const WRITE = LIBPFF_ACCESS_FLAGS_LIBPFF_ACCESS_FLAG_WRITE;
        const READ_WRITE = Self::READ.bits | Self::WRITE.bits;
    }
}

#[cfg(test)]
mod tests {
    use crate::{item::ItemExt, FileOpenFlags, Pff};

    const TEST_PST_FILE: &str = "../data/sample.ost";

    #[test]
    fn pff_new() {
        let _ = Pff::new().unwrap();
    }

    #[test]
    fn file_open() {
        let pff = Pff::new().unwrap();
        let _ = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
    }

    #[test]
    fn non_existent_file() {
        let pff = Pff::new().unwrap();
        let _ = pff
            .open("/this/file/does/not/exist.ost", FileOpenFlags::READ)
            .unwrap_err();
    }

    #[test]
    fn file_size() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let size = pff.size().unwrap().unwrap();
        assert_ne!(size, 0);
    }

    #[test]
    fn root_item() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let _ = pff.root_item().unwrap().unwrap();
    }

    #[test]
    fn root_folder() {
        let pff = Pff::new().unwrap();
        let pff = pff.open(TEST_PST_FILE, FileOpenFlags::READ).unwrap();
        let folder = pff.root_folder().unwrap().unwrap();
        assert!(folder.id().is_ok());
    }
}
