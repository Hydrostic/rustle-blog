

use salvo::http::{ParseError, StatusCode};
use tracing::debug;
use crate::core::config::DEBUG_MODE;
use rustle_derive::NormalError;
#[derive(Debug)]
pub enum AppError {
    ExpectedError(String),
    UnexpectedError(StatusCode, String),
}

pub trait RemapError<T> {
    fn remap(self) -> Result<T, AppError>;
}

impl From<ParseError> for AppError {
    #[tracing::instrument(name="parse_payload",skip_all)]
    fn from(value: ParseError) -> Self {
        if *DEBUG_MODE {
            debug!("failed to parse payload, {:?}", value);
        }
        AppError::UnexpectedError(StatusCode::BAD_REQUEST, String::from("error.request.payload.unrecognizable"))
    }
}
impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        if *DEBUG_MODE {
            debug!("failed to validate payload, {:?}", value);
        }
        AppError::UnexpectedError(StatusCode::BAD_REQUEST, String::from("error.request.payload.invalid"))
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(NormalError)]
pub enum NormalErrorGlobal{
    #[msg = "{_0} not found"]
    NotFound(&'static str),
    #[msg = "credential {_0} unauthorized"]
    UnauthorizedCredential(&'static str),
    #[msg = "status unauthorized"]
    UnauthorizedStatus,
    // #[msg = "feature {_0} not enabled"]
    // FeatureNotEnabled(&'static str),
    #[msg = "unknown lang"]
    UnknownLang,
    #[msg = "permission denied"]
    PermissionDenied
}
// #[macro_export]
// macro_rules! print_error {
//     ($old: expr) => {{
//         let r = $old;
//         if let Err(e) = r {
//             tracing::error!("{:?}(line {})", e, line!());
//         }
//         $old
//     }};
// }