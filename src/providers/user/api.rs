
use crate::core::config::read_config;
use crate::db::{user as userDao, verification as verificationDao, RB};
use crate::middlewares::auth::auth_middleware;
use crate::utils::error_handling::AppResult;
use crate::utils::hmac::hmac_verify;
use crate::utils::password_salt;
use crate::utils::response::{
    NormalResponseGlobal::{FeatureNotEnabled, NotFound, UnauthorizedCredential, UnknownLang},
    ResponseUtil,
};

use fluent_templates::LanguageIdentifier;
use rustle_derive::NormalResponse;
use salvo::prelude::*;
use serde::Deserialize;
use validator::Validate;

use super::service::{send_tolink_email, send_verify_email, SendMailError};
pub fn init() -> Router {
    Router::with_path("/v1/user")
        .push(
            Router::with_hoop(auth_middleware).push(
                Router::with_path("/change_email")
                    .post(change_email)
                    .push(Router::with_path("/change_password").post(change_password)),
            ).push(Router::with_path("/change_password").post(change_password)),
        )
        .push(Router::with_path("/forgot_password").post(forgot_password))
        .push(Router::with_path("/verify_email").post(verify_email))

}

#[derive(NormalResponse)]
pub enum NormalResponseMail {
    #[msg = "mail busy"]
    MailBusy,
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct ChangeEmailReq<'a> {
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
    let req_data = req.parse_json::<ChangeEmailReq>().await?;
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

#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct ChangePasswordReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub old_password: &'a str,
    #[validate(length(min = 1, max = 50))]
    pub new_password: &'a str,
}

#[handler]
async fn change_password(depot: &mut Depot, req: &mut Request, res: &mut Response) -> AppResult {
    let session = depot.session_mut().unwrap();
    let req_data = req.parse_json::<ChangePasswordReq>().await?;
    let user_id: i32 = session.get("user_id").unwrap();
     let user =
        match userDao::select_by_id_with_password(&mut RB.clone(), user_id).await? {
            Some(t) => t,
            None => return res.normal_response(NotFound("user")),
        };
    if !password_salt::compare_password(user.password.as_ref().unwrap(), &req_data.old_password) {
        return res.normal_response(UnauthorizedCredential("password"));
    }
    userDao::update_password(&mut RB.clone(), user_id, &req_data.new_password).await?;
    res.ok()
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct VerifyEmailReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub code: &'a str
}
#[handler]
async fn verify_email(req: &mut Request, res: &mut Response) -> AppResult {
    let req_data = req.parse_json::<VerifyEmailReq>().await?;
    let code_vec: Vec<&str> = req_data.code.split('.').collect();
    if code_vec.len() != 2{
        return res.normal_response(UnauthorizedCredential("code"));
    }
    if !hmac_verify(&read_config().security.credential_secret, code_vec[1], code_vec[0]) {
        return res.normal_response(UnauthorizedCredential("code"));
    }
    let verification_id: i32 = code_vec[1].parse().unwrap();
    let ver = match verificationDao::select_by_id(&mut RB.clone(), verification_id).await? {
        Some(t) => t,
        None => return res.normal_response(UnauthorizedCredential("code")),
    };
    userDao::update_email(&mut RB.clone(), ver.user, &ver.identity).await?;
    verificationDao::delete_by_id(&mut RB.clone(), verification_id).await?;
    res.ok()
}
