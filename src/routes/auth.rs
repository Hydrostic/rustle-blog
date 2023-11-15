
use salvo::prelude::*;
use validator::Validate;
use serde::Deserialize;
use crate::utils::error_handling::AppResult;

pub fn init(router: Router) -> Router {
    router.push(
        Router::with_path("/v1/sign_in").post(sign_in)
    )
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", format = "json")))]
struct SignInReq{
    #[validate(length(min = 1,max = 50))]
    pub name: String,
    #[validate(length(min = 1,max = 50))]
    pub password: String
}
#[handler]
async fn sign_in(req: &mut Request,_res: &mut Response) -> AppResult {
    let req_data = req.parse_json::<SignInReq>().await?;
    req_data.validate()?;
    
    Ok(())
}