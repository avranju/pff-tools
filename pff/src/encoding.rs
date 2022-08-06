use std::borrow::Cow;

use crate::error::Error;

pub(crate) fn to_string(buf: &[u8], code_page: u32) -> Result<Cow<'_, str>, Error> {
    let encoding = codepage::to_encoding(
        code_page
            .try_into()
            .map_err(|_| Error::BadCodePage(code_page))?,
    )
    .ok_or(Error::BadCodePage(code_page))?;

    let (out_str, _, _) = encoding.decode(buf);
    Ok(out_str)
}
