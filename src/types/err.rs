use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use rustle_derive::ErrorHelper;
#[derive(Debug)]
pub enum AppError {
    User(ntex::http::StatusCode, Cow<'static, str>),
    Internal(ntex::http::StatusCode, Cow<'static, str>)
}
impl Display for AppError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.status_code().as_u16(), self.message()))
    }
}
impl AppError {
    pub fn new_user(status: ntex::http::StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        AppError::User(status, message.into())
    }
    pub fn new_internal(status: ntex::http::StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        AppError::Internal(status, message.into())
    }
    pub fn status_code(&self) -> ntex::http::StatusCode {
        match self{
            AppError::User(status, _) => *status,
            AppError::Internal(status, _) => *status
        }
    }

    pub fn message(&self) -> &Cow<str> {
        match self{
            AppError::User(_, message) => message,
            AppError::Internal(_, message) => message
        }
    }
}
pub type AppResult<T> = Result<T, AppError>;
pub type EmptyErrResult<T> = Result<T, ()>;
#[derive(ErrorHelper)]
#[err(default_msg, user)]
pub enum GlobalUserError{
    NotFound,
    CredentialUnauthorized,
    StatusUnauthorized,
    PermissionDenied,
    TooMaxParameter,
    #[err(code = 412)]
    InvalidParameter,
    #[err(code = 400)]
    InvalidPayload,
    #[err(code = 400)]
    InvalidMime,
    UnknownLang,
    #[err(code = 501)]
    FeatureNotEnabled,
    #[err(code = 413)]
    PayloadTooLarge,
    #[err(code = 411)]
    PayloadLengthRequired,
    SystemReserved
}

#[derive(ErrorHelper)]
#[err(internal)]
pub enum GlobalInternalError{
    #[err(msg = "error.io")]
    IO
}