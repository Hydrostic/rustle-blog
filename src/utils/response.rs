
use crate::types::err::AppError;
// use futures_util::{AsyncRead, Stream};
use ntex::web;

impl web::error::WebResponseError for AppError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::build(self.status_code())
            .body(
                serde_json::json!({
                    "message": self.message(),
                    "type": match self {
                        AppError::User(_, _) => "user",
                        AppError::Internal(_, _) => "internal"
                    }
                })
            )
    }

    fn status_code(&self) -> ntex::http::StatusCode {
        self.status_code()
    }
}

