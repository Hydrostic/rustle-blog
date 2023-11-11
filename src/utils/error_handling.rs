use salvo::http::{ParseError, StatusCode};
use salvo::prelude::*;
use serde::Serialize;
use std::panic::Location;
use tracing::error;
use validator;
use crate::core::config;
#[derive(Serialize)]
pub struct AppError {
    pub code: u16,
    pub message: String,
    #[serde(skip)]
    pub source: Option<anyhow::Error>,
    #[serde(skip)]
    pub loc: Option<Location<'static>>,
    pub only_debug_print: bool
}

impl From<anyhow::Error> for AppError {
    #[track_caller]
    fn from(value: anyhow::Error) -> Self {
        let loc = Location::caller();

        AppError {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "core.server_error".to_string(),
            source: Some(value),
            loc: Some(*loc),
            only_debug_print: false
        }
    }
}
impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        AppError {
            code: StatusCode::BAD_REQUEST.as_u16(),
            message: "web.validate_failed".to_string(),
            source: Some(value.into()),
            loc: None,
            only_debug_print: true
        }
    }
}
impl From<ParseError> for AppError {
    fn from(_value: ParseError) -> Self {
        AppError {
            code: StatusCode::BAD_REQUEST.as_u16(),
            message: "core.unrecognizable_request_payload".to_string(),
            source: None,
            loc: None,
            only_debug_print: true
        }
    }
}
pub type AppResult = Result<(), AppError>;

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let mut loc_text = String::from("unknown path");
        if let Some(loc) = self.loc {
            loc_text = format!("{}:{}", loc.file(), loc.line());
        }
        if let Some(ref e) = self.source{
            if *config::DEBUG_MODE || !self.only_debug_print {
                error!(target: "rustle_blog::web", 
                        "\nError happened at {} when requesting {}\n{:?}", 
                        loc_text, 
                        req.uri(),
                        e
                );
            }
        }
        
        res.status_code(StatusCode::from_u16(self.code).unwrap());
        res.render(Json(self));
    }
}
#[macro_export]
macro_rules! print_error {
    ($old: expr) => {{
        let r = $old;
        if let Err(e) = r {
            tracing::error!("{:?}(line {})", e, line!());
        }
        $old
    }};
}