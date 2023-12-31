use crate::communication::mail::{self, MailToLinkTemplate, MailVerifyTemplate, MAILER_ENABLED, MailQueueError};
use crate::core::config::read_config;
use crate::db::{user as userDao, verification as verificationDao, RB};
use crate::middlewares::auth::auth_middleware;
use crate::utils::error_handling::AppResult;
use crate::utils::password_salt;
use crate::utils::response::NormalResponseGlobal;
use crate::utils::response::{
    NormalResponseGlobal::{FeatureNotEnabled, NotFound, UnauthorizedCredential, UnknownLang},
    ResponseUtil,
};
use crate::utils::hmac::hmac_signature;
use anyhow::Context;

use fluent_templates::LanguageIdentifier;
use rand::Rng;
use rustle_derive::NormalResponse;
use salvo::prelude::*;
use serde::Deserialize;
use std::sync::atomic;
use validator::Validate;

use super::service::{send_tolink_email, send_verify_email, SendMailError};
pub fn init() -> Router {
    Router::with_path("/v1/user")
        .push(
            Router::with_path("/change_email")
                .hoop(auth_middleware)
                .post(change_email),
        )
        .push(Router::with_path("/forgot_password").post(forgot_password))
}

#[derive(NormalResponse)]
pub enum NormalResponseMail {
    #[msg = "mail busy"]
    MailBusy,
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct ChangePasswordReq<'a> {
    #[validate(length(min = 3, max = 100))]
    pub email: &'a str,
    #[validate(length(min = 1, max = 50))]
    pub password: &'a str,
    #[validate(length(min = 1, max = 10))]
    pub lang: &'a str,
}

#[handler]
async fn change_email(depot: &mut Depot, req: &mut Request, res: &mut Response) -> AppResult {
    
    let session = depot.session_mut().unwrap();
    let req_data = req.parse_json::<ChangePasswordReq>().await?;
    let li: LanguageIdentifier = match req_data.lang.parse(){
        Err(_) => return res.normal_response(UnknownLang),
        Ok(t) => t
    };
    let user_id: i32 = session.get("user_id").unwrap();
     let user =
        match userDao::select_by_id_with_password(&mut RB.clone(), user_id).await? {
            Some(t) => t,
            None => return res.normal_response(NotFound("user")),
        };
    if !password_salt::compare_password(user.password.as_ref().unwrap(), &req_data.password) {
        return res.normal_response(UnauthorizedCredential("password"));
    }
    match send_tolink_email(req_data.email, &user, "change_email", &li).await{
        Ok(_) => {},
        Err(SendMailError::MailBusy) => return res.normal_response(NormalResponseMail::MailBusy),
        Err(SendMailError::FeatureNotEnabled) => return res.normal_response(FeatureNotEnabled("mail")),
        Err(SendMailError::Unexpected(e)) => return Err(e.into()),
    };
    res.ok()
}

#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct ForgotPasswordReq<'a> {
    #[validate(length(min = 3, max = 100))]
    pub email: &'a str,
    #[validate(length(min = 1, max = 10))]
    pub lang: &'a str,
}
#[handler]
async fn forgot_password(req: &mut Request, res: &mut Response) -> AppResult {
    let req_data = req.parse_json::<ForgotPasswordReq>().await?;
    let li: LanguageIdentifier = match req_data.lang.parse(){
        Err(_) => return res.normal_response(UnknownLang),
        Ok(t) => t
    };
    let user =
        match userDao::select_by_email(&mut RB.clone(), req_data.email).await? {
            Some(t) => t,
            None => return res.normal_response(NotFound("email")),
        };
    
    match send_verify_email(&user, "change_password", &li).await{
        Ok(_) => {},
        Err(SendMailError::MailBusy) => return res.normal_response(NormalResponseMail::MailBusy),
        Err(SendMailError::FeatureNotEnabled) => return res.normal_response(FeatureNotEnabled("mail")),
        Err(SendMailError::Unexpected(e)) => return Err(e.into()),
    };

    res.ok()
}
