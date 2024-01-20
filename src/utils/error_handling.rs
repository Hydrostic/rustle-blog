

use salvo::http::{ParseError, StatusCode};
use salvo::prelude::*;
use tracing::debug;
use crate::core::config::DEBUG_MODE;
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

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, depot: &mut Depot, res: &mut Response) {
        let message = match self{
            AppError::ExpectedError(message) => {
                res.status_code(StatusCode::OK);
                message
            },
            AppError::UnexpectedError(code, message) => {
                res.status_code(code);
                message
            }
        };
        res.render(Text::Json(
            format!(r#"{{"message":"{}","request_id":"{}"}}"#, 
                message, 
                depot.get::<String>("request_id").unwrap_or(&String::from("unknown"))
            )
        ));
    }
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