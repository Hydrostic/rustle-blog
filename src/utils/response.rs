use salvo::{writing::Text, Scribe};
use super::error_handling::{AppResult, AppError};
use rustle_derive::NormalResponse;
use rustle_derive_additional::MessagePrintable;
pub trait ResponseUtil {
    fn ok(&mut self) -> AppResult<()>;
    fn data<P: Scribe>(&mut self, piece: P) -> AppResult<()>;
}

#[derive(NormalResponse)]
pub enum NormalResponseGlobal{
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
    // #[msg = "permission denied"]
    // PermissionDenied
}
pub fn normal_response(res: impl MessagePrintable) -> AppResult<()>{
    Err(AppError::ExpectedError(String::from(res.print())))
}
impl ResponseUtil for salvo::http::Response {
    fn ok(&mut self) -> AppResult<()>{
        
        self.render(Text::Json(r#"{"message":"ok"}"#));
        Ok(())
    }
    
    fn data<P: Scribe>(&mut self, piece: P) -> AppResult<()>{
        self.render(piece);
        Ok(())
    }
}