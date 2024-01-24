use std::ffi::OsString;

pub type Utf8DecodeResult = Result<String, String>;

#[cfg(target_family = "unix")]
pub fn os_string_to_u8_vec(os_string: &OsString) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    os_string.as_bytes().to_vec()
}

#[cfg(target_os = "wasi")]
pub fn os_string_to_u8_vec(os_string: &OsString) -> Vec<u8> {
    use std::os::wasi::ffi::OsStrExt;
    os_string.as_bytes().to_vec()
}

#[cfg(any(target_family = "unix", target_os = "wasi"))]
pub fn u8_vec_to_string(bytes: &[u8]) -> Utf8DecodeResult {
    std::str::from_utf8(bytes).map_or_else(
        |_| {
            Err(bytes
                .iter()
                .map(|c| {
                    let escaped = std::ascii::escape_default(*c).collect::<Vec<_>>();
                    String::from_utf8(escaped).unwrap()
                })
                .collect())
        },
        |str| Ok(str.to_owned()),
    )
}

#[cfg(target_family = "windows")]
pub fn os_string_to_u8_vec(os_string: &OsString) -> Vec<u8> {
    // On Windows, OsString must be valid Unicode
    os_string.to_str().unwrap().as_bytes().to_vec()
}

#[cfg(target_family = "windows")]
#[allow(clippy::unnecessary_wraps)]
pub fn u8_vec_to_string(bytes: &[u8]) -> Utf8DecodeResult {
    // On Windows, OsString must be valid Unicode
    Ok(std::str::from_utf8(bytes).unwrap().into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode() {
        let bytes = b"abc";
        let formatted = u8_vec_to_string(bytes);
        assert_eq!(formatted, Ok("abc".to_string()));
    }

    #[test]
    #[cfg(any(target_family = "unix", target_os = "wasi"))]
    fn unix_decode_non_utf8() {
        let bytes = vec![0x54, 0x65, 0x73, 0x74, 0xc3, 0x28];
        let formatted = u8_vec_to_string(&bytes);
        assert_eq!(formatted, Err("Test\\xc3(".to_string()));
    }
}
