use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use salvo::http::StatusCode;

use rustle_derive::ErrorHelper;
#[derive(Debug)]
pub enum AppError {
    User(StatusCode, Cow<'static, str>),
    Internal(StatusCode, Cow<'static, str>)
}
impl Display for AppError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.status_code().as_u16(), self.message()))
    }
}
impl AppError {
    pub fn new_user(status: StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        AppError::User(status, message.into())
    }
    pub fn new_internal(status: StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        AppError::Internal(status, message.into())
    }
    pub fn status_code(&self) -> StatusCode {
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
    UnknownLang,
    #[err(code = 501)]
    FeatureNotEnabled,
    #[err(code = 413)]
    PayloadTooLarge,
    #[err(code = 411)]
    PayloadLengthRequired
}

#[derive(ErrorHelper)]
#[err(internal)]
pub enum GlobalInternalError{
    #[err(msg = "error.io")]
    IO
}