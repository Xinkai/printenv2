#[derive(Debug)]
pub enum AppError {
    Utf8Error(std::str::Utf8Error),
    StdIo(std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<std::str::Utf8Error> for AppError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::StdIo(err)
    }
}
