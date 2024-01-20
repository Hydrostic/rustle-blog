use fluent_templates::{Loader, LanguageIdentifier};
use rand::Rng;
use salvo::hyper::StatusCode;
use tracing::error;

use crate::core::config::read_config;
use crate::communication::mail::{self, MailToLinkTemplate, MailVerifyTemplate, MAILER_ENABLED, MailQueueError};
use crate::db::user::User;
use crate::db::{verification as verificationDao, get_db_pool};
use crate::fs::embed::LOCALES;
use crate::utils::error_handling::{AppResult, AppError};
use crate::utils::hmac::hmac_signature;
use std::sync::atomic;


pub async fn send_tolink_email(email: &str, user: &User, action: &str, lang: &LanguageIdentifier) -> AppResult<()>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(AppError::UnexpectedError(StatusCode::NOT_IMPLEMENTED, String::from("error.mail.not_enabled")));
    }
    let verification_res = verificationDao::create(get_db_pool(), user.id, &user.email, "", 0).await?;
    let mut sig = hmac_signature(
        &read_config().security.credential_secret,
        &verification_res.to_string(),
    );
    sig.push('.');
    sig.push_str(&verification_res.to_string());
    let mail_content = MailToLinkTemplate {
        site_name: read_config().info.name.clone(),
        site_link: read_config().info.link.clone(),
        user: user.name.clone(),
        link: sig,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| {
        error!("failed to generate mail content, {:?}", x);
        AppError::UnexpectedError(StatusCode::INTERNAL_SERVER_ERROR, String::from("error.mail.render"))
    })?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "email_check").unwrap().as_str(), 
        &user.name,
        &email){
        Err(MailQueueError::MailBusy) => return Err(
            AppError::UnexpectedError(StatusCode::SERVICE_UNAVAILABLE, String::from("error.mail.busy"))
        ),
        Err(MailQueueError::ParseError) => return Err(
            AppError::UnexpectedError(StatusCode::SERVICE_UNAVAILABLE, String::from("error.mail.parse"))
        ),
        Ok(_) => {}
    };
    Ok(())
}

pub async fn send_verify_email(user: &User, action: &str, lang: &LanguageIdentifier) -> AppResult<()>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(AppError::UnexpectedError(StatusCode::NOT_IMPLEMENTED, String::from("error.mail.not_enabled")));
    }

    let code_chars: Vec<char> = (0..6)
    .map(|_| rand::thread_rng().gen_range('0'..'9'))
    .collect();
    let code: String = code_chars.into_iter().collect();
    _ = verificationDao::create(get_db_pool(), user.id, &user.email, &code, 0).await?;
    let mail_content = MailVerifyTemplate {
        site_name: read_config().info.name.clone(),
        site_link: read_config().info.link.clone(),
        user: user.name.clone(),
        code,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| {
        error!("failed to generate mail content, {:?}", x);
        AppError::UnexpectedError(StatusCode::INTERNAL_SERVER_ERROR, String::from("error.mail.render"))
    })?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "identity_check").unwrap().as_str(), 
        &user.name,
        &user.email){
            Err(MailQueueError::MailBusy) => return Err(
                AppError::UnexpectedError(StatusCode::SERVICE_UNAVAILABLE, String::from("error.mail.busy"))
            ),
            Err(MailQueueError::ParseError) => return Err(
                AppError::UnexpectedError(StatusCode::SERVICE_UNAVAILABLE, String::from("error.mail.parse"))
            ),
        Ok(_) => {}
    };
    Ok(())
}