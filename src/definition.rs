#[derive(Debug)]
pub enum AppError {
    Utf8Error(std::str::Utf8Error),
    StdIo(std::io::Error),
    #[cfg(windows)]
    WindowsCore(windows::core::Error),
    #[cfg(windows)]
    Utf16Error(std::string::FromUtf16Error),

    #[cfg(unix_kvm)]
    UnixErrorString(std::ffi::CString),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<std::str::Utf8Error> for AppError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

#[cfg(windows)]
impl From<std::string::FromUtf16Error> for AppError {
    fn from(err: std::string::FromUtf16Error) -> Self {
        Self::Utf16Error(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::StdIo(err)
    }
}

#[cfg(windows)]
impl From<windows::core::Error> for AppError {
    fn from(err: windows::core::Error) -> Self {
        Self::WindowsCore(err)
    }
}

#[cfg(unix_kvm)]
impl From<std::ffi::CString> for AppError {
    fn from(err_str: std::ffi::CString) -> Self {
        Self::UnixErrorString(err_str)
    }
}
