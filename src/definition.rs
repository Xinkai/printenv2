#[derive(Debug)]
pub enum AppError {
    Utf8Error(std::str::Utf8Error),
    StdIo(std::io::Error),
    SerdeJson(serde_json::Error),

    #[cfg(windows)]
    WindowsCore(windows::core::Error),
    #[cfg(windows)]
    Utf16Error(std::string::FromUtf16Error),
    #[cfg(windows)]
    NulError(std::ffi::NulError),

    #[cfg(unix_kvm)]
    UnixErrorString(std::ffi::CString),
    #[cfg(unix_kvm)]
    NulError(std::ffi::NulError),

    #[cfg(any(unix_kvm, unix_apple_sysctl))]
    TryFromIntError(std::num::TryFromIntError),

    #[cfg(unix_apple_sysctl)]
    TryFromSliceError(std::array::TryFromSliceError),
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

#[cfg(windows)]
impl From<std::ffi::NulError> for AppError {
    fn from(err: std::ffi::NulError) -> Self {
        Self::NulError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::StdIo(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
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

#[cfg(unix_kvm)]
impl From<std::ffi::NulError> for AppError {
    fn from(err: std::ffi::NulError) -> Self {
        Self::NulError(err)
    }
}

#[cfg(any(unix_kvm, unix_apple_sysctl))]
impl From<std::num::TryFromIntError> for AppError {
    fn from(err: std::num::TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

#[cfg(unix_apple_sysctl)]
impl From<std::array::TryFromSliceError> for AppError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        Self::TryFromSliceError(err)
    }
}
