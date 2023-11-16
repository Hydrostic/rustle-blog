use salvo::writing::Text;
use super::error_handling::AppResult;
use rustle_derive::NormalResponse;
use rustle_derive_additional::MessagePrintable;
pub trait ResponseUtil {
    fn ok(&mut self) -> AppResult;
    fn normal_response(&mut self, res: impl MessagePrintable) -> AppResult;
}

#[derive(NormalResponse)]
pub enum NormalResponseGlobal{
    #[msg = "{_0} not found"]
    NotFound(&'static str),
    #[msg = "credential {_0} unauthorized"]
    UnauthorizedCredential(&'static str),
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
}