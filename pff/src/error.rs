use std::{ffi::CStr, fmt::Display};

use pff_sys::{libpff_error_free, libpff_error_sprint, libpff_error_t};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("{0}")]
    PffError(#[source] PffError),

    #[error("{0}")]
    NulError(#[from] std::ffi::NulError),

    #[error("{0}")]
    FromVecWithNulError(#[from] std::ffi::FromVecWithNulError),

    #[error("Couldn't convert UTF16 encoded bytes to String")]
    FromUtf16Error(#[from] std::string::FromUtf16Error),

    #[error("{0}")]
    IntoStringError(#[from] std::ffi::IntoStringError),

    #[error("Unrecognized item type {0}")]
    BadItemType(u8),

    #[error("Unrecognized entry type {0}")]
    BadEntryType(u32),

    #[error("Unrecognized value type {0}")]
    BadValueType(u32),

    #[error("Bad UUID")]
    BadUuid(#[from] uuid::Error),

    #[error("Item is not a folder.")]
    NotAFolder,

    #[error("Codepage {0} is not supported.")]
    BadCodePage(u32),
}

impl Error {
    pub fn pff_error(error: *mut libpff_error_t) -> Self {
        Error::PffError(PffError::new(error))
    }
}

#[derive(Debug, ThisError)]
pub struct PffError {
    error: *mut libpff_error_t,
}

impl PffError {
    pub fn new(error: *mut libpff_error_t) -> Self {
        Self { error }
    }
}

unsafe impl Send for PffError {}

// TODO: Is this safe?
unsafe impl Sync for PffError {}

impl Drop for PffError {
    fn drop(&mut self) {
        unsafe { libpff_error_free(&mut self.error) };
    }
}

impl Display for PffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::<i8>::with_capacity(1024);
        let buf_ptr = buf.as_mut_ptr();

        let res = unsafe { libpff_error_sprint(self.error, buf_ptr, buf.capacity() as u64) };
        match res {
            -1 => write!(f, "PFF error"),
            _ => {
                let c_str: &CStr = unsafe { CStr::from_ptr(buf_ptr) };
                match c_str.to_str() {
                    Ok(s) => write!(f, "{}", s),
                    Err(_) => write!(f, "PFF error"),
                }
            }
        }
    }
}
