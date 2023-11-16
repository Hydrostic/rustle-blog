
use salvo::prelude::*;
use validator::Validate;
use serde::Deserialize;
use crate::db::{user as userDao, RB};
use crate::utils::error_handling::AppResult;
use crate::utils::response::{ResponseUtil, NormalResponseGlobal::{NotFound, UnauthorizedCredential}};


pub fn init(router: Router) -> Router {
    
    // #[cfg(debug_assertions)]
    // router.push(
    //     Router::with_path("/v1/auth/__test_add_user__").post(sign_in)
    // );
    router
    .push(
        Router::with_path("/v1/auth/sign_in").post(sign_in)
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
async fn sign_in(req: &mut Request,res: &mut Response) -> AppResult {
    let req_data = req.parse_json::<SignInReq>().await?;
    req_data.validate()?;
    let user_data = userDao::select_by_identity(&mut RB.clone(), &req_data.name).await?;
    if let None = user_data{
        return res.normal_response(NotFound("user"))
    }
    let user_data = user_data.unwrap();
    if !userDao::compare_password(&user_data.password, &req_data.password){
        return res.normal_response(UnauthorizedCredential("name/password"))
    }
    res.ok()
}