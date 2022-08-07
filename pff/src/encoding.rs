use std::borrow::Cow;

use crate::error::Error;

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
            let (out_str, _, _) = encoding.decode(buf);
            out_str
        }
    })
}
