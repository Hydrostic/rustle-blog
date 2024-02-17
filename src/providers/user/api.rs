use super::service::{send_tolink_email, send_verify_email};
use crate::db::rbac::{RoleSimple, UserRoleSimple};
use crate::db::user::User;
use crate::db::{get_db_pool, rbac as rbacDao, user as userDao, verification as verificationDao};
use crate::external::fs::interface::FsProvider;
use crate::external::fs::DEFAULT_POLICY_ID;
use crate::get_config;
use crate::middlewares::Auth;
use crate::providers::auth::service::check_permission_api;
use crate::types::err::GlobalUserError::{
    CredentialUnauthorized, NotFound, TooMaxParameter, UnknownLang
};
use crate::types::err::{AppResult, GlobalInternalError};
use crate::utils::hmac::hmac_verify;
use crate::utils::{password_salt, sniffer};
use crate::utils::request::{check_content_length, check_mime, get_user_id, RequestPayload, ALLOWED_IMAGE_MIME};
use fluent_templates::LanguageIdentifier;
use futures_util::TryStreamExt;
use crate::utils::stream::{AsyncReadMerger, ReaderChunkedStream};
use ntex::web::{self, Responder};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use std::borrow::Cow;
use std::io::Cursor;
use std::path::Path;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/v1/user")
            .service(verify_email)
            .service(get_avatar)
            .service(
                web::scope("/").wrap(Auth)
                .service(change_email)
                .service(change_password) 
                .service(get_all_list) 
                .service(upload_avatar)
            )
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

    let user_id: i32 = get_user_id(&req);
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
    let user_id: i32 = get_user_id(&req);
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
#[web::post("/verify_email")]
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
    pub role_infos: Vec<RoleSimple>,
}
#[web::post("/get_all_list")]
async fn get_all_list(mut payload: web::types::Payload, req: web::HttpRequest) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data = payload.parse::<ListReq>().await?;
    req_data.validate()?;

    let user_id = get_user_id(&req);
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
                let role_single = roles
                .iter()
                .filter(|r| r.user == uid).collect::<Vec<&UserRoleSimple>>();
                ListRes {
                    user: t,
                    role_infos: if role_single.len() > 0 {
                        role_single[0].roles.clone()
                    } else {
                        vec![]
                    },
                        
                }
            })
            .collect::<Vec<ListRes>>(),
    ))
}

#[web::post("/upload_avatar")]
pub async fn upload_avatar(payload: web::types::Payload, 
    req: web::HttpRequest) -> AppResult<impl Responder> {
    check_content_length(&req, get_config!(http).max_upload_size)?;
    let _ = check_mime(&req, &ALLOWED_IMAGE_MIME)?;
    
    let mut stream = StreamReader::new(payload.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e) 
    }));
    FsProvider::upload_file_internal(&mut stream, 
        DEFAULT_POLICY_ID,
        get_user_id(&req),
        &Path::new("__user/avatar")
    ).await?;
    Ok(web::HttpResponse::Ok().finish())
}

#[web::get("/avatar/{user_id}")]
pub async fn get_avatar(path: web::types::Path<u32>) -> AppResult<impl Responder> {
    let mut file = FsProvider::get_file(DEFAULT_POLICY_ID, 
        path.into_inner() as i32, &Path::new("__user/avatar")).await?;
    let mut header = file.read_i64().await.map_err(|e| {
        tracing::error!("read i64 from file error: {}", e);
        GlobalInternalError::IO
    })?.to_ne_bytes();
    header.reverse();
    let ext_str: &'static str = sniffer::image_sniff(header).into();
    let stream = ReaderChunkedStream::new(
        AsyncReadMerger::new(Cursor::new(header), file)
    );
    
    Ok(web::HttpResponse::Ok().header("content-type", format!("image/{ext_str}"))
        .streaming(stream))
}
