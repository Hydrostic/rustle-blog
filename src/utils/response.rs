use salvo::{writing::Text, Scribe};
use super::error_handling::{AppResult, AppError};
use rustle_derive::NormalResponse;
use rustle_derive_additional::MessagePrintable;
pub trait ResponseUtil {
    fn ok(&mut self) -> AppResult;
    fn normal_response(&mut self, res: impl MessagePrintable) -> AppResult;
    fn data<P: Scribe>(&mut self, piece: P) -> AppResult;
}

#[derive(NormalResponse)]
pub enum NormalResponseGlobal{
    #[msg = "{_0} not found"]
    NotFound(&'static str),
    #[msg = "credential {_0} unauthorized"]
    UnauthorizedCredential(&'static str),
    #[msg = "status unauthorized"]
    UnauthorizedStatus,
    #[msg = "feature {_0} not enabled"]
    FeatureNotEnabled(&'static str),
}
impl ResponseUtil for salvo::http::Response {
    fn ok(&mut self) -> AppResult{
        
        self.render(Text::Json(r#"{"message":"ok"}"#));
        Ok(())
    }
    fn normal_response(&mut self, res: impl MessagePrintable) -> AppResult{
        self.render(Text::Json(format!("{{\"message\":\"{}\"}}", res.print())));
        Ok(())
    }
    fn data<P: Scribe>(&mut self, piece: P) -> AppResult{
        self.render(piece);
        Ok(())
    }
}