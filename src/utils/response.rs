use salvo::{writing::Text, Scribe, Writer, async_trait, Request, Depot, Response, http::StatusCode};
use super::error_handling::{AppResult, AppError};

pub trait ResponseUtil {
    fn ok(&mut self) -> AppResult<()>;
    fn data<P: Scribe>(&mut self, piece: P) -> AppResult<()>;
}


#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, depot: &mut Depot, res: &mut Response) {
        render_error(self, depot, res);
    }
}

pub trait NormalErrorHelper: Sized{
    fn to_error(&self) -> AppError;
}
pub fn render_error(err: AppError, depot: &mut Depot, res: &mut Response){
    let message = match err{
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