use std::{borrow::Cow, ffi::CString};

use crate::{
    error::Error,
    item::{EntryType, Item, ItemExt},
};

pub(crate) fn to_string(buf: &[u8], code_page: u32) -> Result<Cow<'_, str>, Error> {
    Ok(match code_page {
        // According to https://docs.microsoft.com/en-us/windows/win32/intl/code-page-identifiers
        // 20127 is 7-bit ASCII but the codepage crate doesn't support it for
        // some reason. So try to decode it as UTF-8.
        20127 => encoding_rs::mem::decode_latin1(buf),
        code_page => {
            let encoding = codepage::to_encoding(
                code_page
                    .try_into()
                    .map_err(|_| Error::BadCodePage(code_page))?,
            )
            .ok_or(Error::BadCodePage(code_page))?;

            // strings in pst/ost files seem to have some leading control characters
            // for some reason; we trim those
            let (index, _) = buf
                .iter()
                .enumerate()
                .find(|(_, c)| !(**c as char).is_control())
                .unwrap_or((0, &0));

            let (out_str, _, _) = encoding.decode(&buf[index..]);
            out_str
        }
    })
}

pub(crate) fn try_get_item_string<T: Item>(
    item: &T,
    entry_type: EntryType,
    buf: Vec<u8>,
) -> Result<String, Error> {
    match item.first_entry_by_type(entry_type)? {
        Some(code_page) => Ok(to_string(&buf, code_page.as_u32()?)?.to_string()),
        None => Ok(CString::from_vec_with_nul(buf)?.into_string()?),
    }
}
