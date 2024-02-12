use super::service::{send_tolink_email, send_verify_email};
use crate::db::rbac::RoleSimple;
use crate::db::user::User;
use crate::db::{get_db_pool, rbac as rbacDao, user as userDao, verification as verificationDao};
use crate::external::fs::interface::FsProvider;
use crate::get_config;
use crate::middlewares::Auth;
use crate::providers::auth::service::check_permission_api;
use crate::types::err::GlobalUserError::{
    CredentialUnauthorized, NotFound, TooMaxParameter, UnknownLang,
};
use crate::types::err::AppResult;
use crate::utils::hmac::hmac_verify;
use crate::utils::password_salt;
use crate::utils::request::{get_user_id, RequestPayload};
use fluent_templates::LanguageIdentifier;
use futures_util::TryStreamExt;
use ntex::web::{self, Responder};
use serde::{Deserialize, Serialize};
use tokio_util::io::StreamReader;
use std::borrow::Cow;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/v1/user")
            .service(verify_email)
            .service(upload_avatar)
            .configure(|r|{
                r.service(
                    web::scope("/").wrap(Auth)
                    .service(change_email) // auth
                    .service(change_password) // auth
                    .service(get_all_list) // auth
                );
            })
            
            
    );
}

#[derive(Debug, Validate, Deserialize)]
struct ChangeEmailReq<'a> {
    #[validate(length(min = 3, max = 100))]
    pub email: &'a str, // as email should not contain any special characters, it's ok to use raw str
    #[validate(length(min = 1, max = 50))]
    pub password: Cow<'a, str>,
    #[validate(length(min = 1, max = 10))]
    pub lang: &'a str,
}

#[web::post("/change_email")]
async fn change_email(mut payload: web::types::Payload, req: web::HttpRequest) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<ChangeEmailReq>().await?;
    let li: LanguageIdentifier = req_data.lang.parse().map_err(|_| UnknownLang)?;

    let user_id: i32 = get_user_id(req);
    let user = userDao::select_by_id_with_password(get_db_pool(), user_id)
        .await?
        .ok_or(NotFound)?;
    if !password_salt::compare_password(user.password.as_ref().unwrap(), &req_data.password) {
        return Err(CredentialUnauthorized.into());
    }
    send_tolink_email(req_data.email, &user, "change_email", &li).await?;
    Ok(web::HttpResponse::Ok().finish())
}

#[derive(Debug, Validate, Deserialize)]
struct ForgotPasswordReq<'a> {
    #[validate(length(min = 3, max = 100))]
    pub email: &'a str,
    #[validate(length(min = 1, max = 10))]
    pub lang: &'a str,
}
#[web::post("/forgot_password")]
async fn forgot_password(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<ForgotPasswordReq>().await?;
    let li: LanguageIdentifier = req_data.lang.parse().map_err(|_| UnknownLang)?;
    let user = userDao::select_by_email(get_db_pool(), req_data.email)
        .await?
        .ok_or(NotFound)?;
    send_verify_email(&user, "change_password", &li).await?;
    Ok(web::HttpResponse::Ok().finish())
}

#[derive(Debug, Validate, Deserialize)]
struct ChangePasswordReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub old_password: &'a str,
    #[validate(length(min = 1, max = 50))]
    pub new_password: &'a str,
}

#[web::post("/change_password")]
async fn change_password(mut payload: web::types::Payload, req: web::HttpRequest) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<ChangePasswordReq>().await?;
    let user_id: i32 = get_user_id(req);
    let user = userDao::select_by_id_with_password(get_db_pool(), user_id)
        .await?
        .ok_or(NotFound)?;
    if !password_salt::compare_password(user.password.as_ref().unwrap(), &req_data.old_password) {
        return Err(CredentialUnauthorized.into());
    }
    userDao::update_password(get_db_pool(), user_id, &req_data.new_password).await?;
    Ok(web::HttpResponse::Ok().finish())
}
#[derive(Debug, Validate, Deserialize)]
struct VerifyEmailReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub code: &'a str,
}
#[web::post("/v1/verify_email")]
async fn verify_email(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<VerifyEmailReq>().await?;
    let code_vec: Vec<&str> = req_data.code.split('.').collect();
    if code_vec.len() != 2 {
        return Err(CredentialUnauthorized.into());
    }
    if !hmac_verify(
        &get_config!(security).credential_secret,
        code_vec[1],
        code_vec[0],
    ) {
        return Err(CredentialUnauthorized.into());
    }
    let verification_id: i32 = code_vec[1].parse().unwrap();
    let ver = verificationDao::select_by_id(get_db_pool(), verification_id)
        .await?
        .ok_or(CredentialUnauthorized)?;
    userDao::update_email(get_db_pool(), ver.user, &ver.identity).await?;
    verificationDao::delete_by_id(get_db_pool(), verification_id).await?;
    Ok(web::HttpResponse::Ok().finish())
}

#[derive(Debug, Validate, Deserialize)]
struct ListReq {
    #[validate(range(min = 0, max = 100))]
    limit: i32,
    #[validate(range(min = 1,))]
    page: i32,
}
#[derive(Serialize)]
struct ListRes {
    #[serde(flatten)]
    pub user: User,
    pub roles: Vec<RoleSimple>,
}
#[web::post("/get_all_list")]
async fn get_all_list(mut payload: web::types::Payload, req: web::HttpRequest) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<ListReq>().await?;
    req_data.validate()?;

    let user_id = get_user_id(req);
    check_permission_api(Some(user_id), "MANAGE_USER").await?;

    let users = userDao::get_list(
        get_db_pool(),
        req_data.limit,
        (req_data.page - 1)
            .checked_mul(req_data.limit)
            .ok_or(TooMaxParameter)?,
    )
    .await?;
    let roles =
        rbacDao::select_user_role_info(get_db_pool(), users.iter().map(|t| t.id).collect()).await?;

    Ok(web::HttpResponse::Ok().json(
        &users
            .into_iter()
            .map(|t| {
                let uid = t.id;
                ListRes {
                    user: t,
                    roles: roles
                        .iter()
                        .filter(|r| r.user == uid)
                        .map(|r| RoleSimple {
                            id: r.id,
                            name: r.name.to_owned(),
                        })
                        .collect(),
                }
            })
            .collect::<Vec<ListRes>>(),
    ))
}

#[web::post("/upload_avatar")]
pub async fn upload_avatar(payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut stream = StreamReader::new(payload.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e) 
    }));
    FsProvider::upload_file(&mut stream, 1, "", "avatar-10.png").await?;
    Ok(web::HttpResponse::Ok().finish())
}
