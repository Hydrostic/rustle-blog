use crate::communication::mail::{MailToLinkTemplate,TX,MAILER_ENABLED};
use crate::core::config::SETTINGS;
use crate::middlewares::auth::auth_middleware;
use crate::utils::error_handling::AppResult;
use crate::utils::response::{NormalResponseGlobal::{FeatureNotEnabled,NotFound}, ResponseUtil};
use std::sync::atomic;
use rustle_derive::NormalResponse;
use salvo::prelude::*;
use serde::Deserialize;
use validator::Validate;
use crate::db::{user as userDao, RB};
pub fn init() -> Router {
    Router::with_path("/v1/user").push(
        Router::with_path("/send_verify_mail")
            .hoop(auth_middleware)
            .post(send_verify_mail),
    )
    .push(
        Router::with_path("/send_forgot_mail")
            .post(send_forgot_mail)
    )
}

#[handler]
async fn send_verify_mail(_req: &mut Request, res: &mut Response) -> AppResult {
    res.ok()
}
#[derive(NormalResponse)]
pub enum NormalResponseMail{
    #[msg = "mail busy"]
    MailBusy,
}
#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", format = "json")))]
struct SendForgotMailReq {
    #[validate(length(min = 3, max = 255))]
    pub email: String,
}
#[handler]
async fn send_forgot_mail(req: &mut Request, res: &mut Response) -> AppResult {
    let req_data = req.parse_json::<SendForgotMailReq>().await?;
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return res.normal_response(FeatureNotEnabled("mail"));
    }
    let user = match userDao::select_id_name_by_email(&mut RB.clone(), req_data.email.as_str()).await?{
        Some(t) => t,
        None => return res.normal_response(NotFound("user"))
    };
    let content = MailToLinkTemplate {
        site_name: (*SETTINGS.read().unwrap()).info.name.clone(),
        greeting: t!("email.greeting", locale = "zh-CN", name = user.name),
        probably_action: t!(
            "email.probably_action",
            locale = "zh-CN",
            action = t!("user.change_password", locale = "zh-CN")
        ),
        warning: t!("email.warning_unauthorized", locale = "zh-CN"),
        link: (*SETTINGS.read().unwrap()).info.link.clone(),
        action: t!("user.change_password", locale = "zh-CN"),
        to: "Hydrostic <hydrostic@outlook.com>".to_string(),
        subject: t!("user.change_password", locale = "zh-CN")
    };
    if let Err(_) = TX.get().clone().unwrap().try_send(Box::new(content)){
        return res.normal_response(NormalResponseMail::MailBusy);
    }
    res.ok()
}
