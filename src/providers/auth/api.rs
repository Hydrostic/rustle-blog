use crate::db::{user as userDao, get_db_pool};
use crate::middlewares::auth::auth_middleware;
use crate::utils::error_handling::AppResult;
use crate::utils::password_salt;
use crate::utils::response::normal_response;
use crate::utils::response::{
    NormalResponseGlobal::UnauthorizedCredential,
    ResponseUtil,
};
use rustle_derive::handler_with_instrument;
use salvo::prelude::*;
use salvo::session::Session;
use serde::Deserialize;
use validator::Validate;

pub fn init() -> Router {
    #[cfg(not(debug_assertions))]
    compile_error!("test method shouldn't be added in release");
        Router::with_path("/v1/auth")
            .push(Router::with_path("/sign_in").post(sign_in))
            .push(Router::with_path("/__test_add_user__").post(__test_add_user__))
            .push(
                Router::with_path("/sign_out")
                    .hoop(auth_middleware)
                    .post(sign_out),
            )
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct SignInReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub name: &'a str,
    #[validate(length(min = 1, max = 50))]
    pub password: &'a str,
}

#[handler_with_instrument]
async fn sign_in(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let req_data = req.parse_json::<SignInReq>().await?;
    req_data.validate()?;
    let user_data = userDao::select_by_identity_with_password(get_db_pool(), req_data.name).await?;
    if user_data.is_none() {
        return normal_response(UnauthorizedCredential("name/password"));
    }

    let user_data = user_data.unwrap();
    if !password_salt::compare_password(&user_data.password.unwrap(), req_data.password) {
        return normal_response(UnauthorizedCredential("name/password"));
    }
    let mut session = Session::new();
    session.insert("user_id", user_data.id).unwrap();
    depot.set_session(session);
    res.ok()
}
#[handler_with_instrument]
async fn sign_out(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let session = depot.session_mut().unwrap();
    session.remove("user_id");
    res.ok()
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct TestAddUser<'a> {
    #[validate(length(min = 1, max = 50))]
    pub name: &'a str,
    #[validate(length(min = 1, max = 100))]
    pub email: &'a str,
    #[validate(length(min = 1, max = 50))]
    pub password: &'a str,
}
#[handler_with_instrument]
async fn __test_add_user__(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let req_data = req.parse_json::<TestAddUser>().await?;
    req_data.validate()?;
    let hashed_password = password_salt::generate_password(req_data.password)?;
    let user_insert_res = userDao::create(
        get_db_pool(),
        req_data.name,
        req_data.email,
        &hashed_password,
        0,
    )
    .await?;
    res.data(Text::Json(format!(
        "{{\"id\":\"{}\"}}",
        user_insert_res
    )))
}
