use std::sync;
use ntex::web::{self, Responder};
use crate::types::err::GlobalUserError::CredentialUnauthorized;
use crate::db::{user as userDao, get_db_pool};
use crate::utils::request::RequestPayload;
use crate::utils::{paseto, password_salt};
use serde::Deserialize;
use std::borrow::Cow;
use serde_json::json;
use validator::Validate;
use crate::{get_args, get_config};
use crate::types::err::AppResult;
use crate::utils::paseto::generate_access_token;

pub static ONCE_INIT: sync::Once = sync::Once::new();
pub fn init(cfg: &mut web::ServiceConfig){
        cfg.service(
            web::scope("/v1/auth")
                .service(sign_in)
                .configure(|r| {
                    if get_args!(debug){
                        r.service(__test_add_user__)
                        .service(__test_get_token__);
                    }
                })
        );
        if get_args!(debug) {
            ONCE_INIT.call_once(|| {
                println!("\x1b[31m!!!@@@ warning @@@!!!\n\
                        debug mode api(__test_add_user__, __test_get_token__) added\n\
                        in no case should you enable it in the production\n\
                        !!!!!!!!!!!!!!!!!!!!!!\x1b[0m");
            });
        }
}
#[derive(Debug, Validate, Deserialize)]
struct SignInReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub name: Cow<'a, str>,
    #[validate(length(min = 1, max = 100))]
    pub password: Cow<'a, str>,
}
#[web::post("/sign_in")]
async fn sign_in(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: SignInReq<'_> = payload.parse().await?;
    req_data.validate()?;
    let user_data = userDao::select_by_identity_with_password(get_db_pool(), &req_data.name).await?;
    if user_data.is_none() {
        return Err(CredentialUnauthorized.into());
    }

    let user_data = user_data.unwrap();
    if !password_salt::compare_password(&user_data.password.unwrap(), &req_data.password) {
        return Err(CredentialUnauthorized.into());
    }
    let token = paseto::generate_access_token(&get_config!(security).auth_token_secret, user_data.id)?;
    Ok(web::HttpResponse::Ok().body(
        json!({
            "id": user_data.id,
            "name": user_data.name,
            "email": user_data.email,
            "access_token": token
        })
    ))
}
// #[web::get("/sign_out")]
// async fn sign_out(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
//     let session = depot.session_mut().unwrap();
//     session.remove("user_id");
//     res.ok()
// }
#[derive(Debug, Validate, Deserialize)]
struct TestAddUser<'a> {
    #[validate(length(min = 1, max = 50))]
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[validate(length(min = 1, max = 100))]
    pub email: &'a str, // as email should not contain any special characters, it's ok to use raw str
    #[validate(length(min = 1, max = 100))]
    #[serde(borrow)]
    pub password: Cow<'a, str>,
}
// debug mode only
#[web::post("/__test_add_user__")]
async fn __test_add_user__(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: TestAddUser<'_> = payload.parse().await?;
    req_data.validate()?;
    let hashed_password = password_salt::generate_password(&req_data.password, &get_config!(security).password_salt)?;
    let user_insert_res = userDao::create(
        get_db_pool(),
        &req_data.name,
        &req_data.email,
        &hashed_password
    )
    .await?;
    Ok(web::HttpResponse::Ok().body(json!({
        "id": user_insert_res
    })))
}
#[derive(Deserialize)]
struct TestGetTokenReq {
    pub id: i32
}
#[web::get("/__test_get_token__")]
async fn __test_get_token__(user: web::types::Query<TestGetTokenReq>) -> AppResult<impl Responder> {
    let token = generate_access_token(&get_config!(security).auth_token_secret, user.id)?;
    Ok(web::HttpResponse::Ok().body(
        json!({
            "access_token": token
        })
    ))
}