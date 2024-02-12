use fluent_templates::{Loader, LanguageIdentifier};
use rand::Rng;
use tracing::error;

use crate::external::mail::{self, MailToLinkTemplate, MailVerifyTemplate, MAILER_ENABLED, MailQueueError};
use crate::db::user::User;
use crate::db::{verification as verificationDao, get_db_pool};
use crate::external::fs::embed::LOCALES;
use crate::types::err::AppResult;
use crate::utils::hmac::hmac_signature;
use std::sync::atomic;
use rustle_derive::ErrorHelper;
use crate::get_config;
use crate::types::err::GlobalUserError::FeatureNotEnabled;

#[derive(ErrorHelper)]
#[err(internal)]
pub enum MailInternalError{
    #[err(msg = "error.mail.render")]
    Render,
    #[err(msg = "error.mail.busy")]
    Busy,
    #[err(msg = "error.mail.parse")]
    Parse
}
pub async fn send_tolink_email(email: &str, user: &User, action: &str, lang: &LanguageIdentifier) -> AppResult<()>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(FeatureNotEnabled.into());
    }
    let verification_res = verificationDao::create(get_db_pool(), user.id, &user.email, "", 0).await?;
    let mut sig = hmac_signature(
        &get_config!(security).credential_secret,
        &verification_res.to_string(),
    );
    sig.push('.');
    sig.push_str(&verification_res.to_string());
    let site_info = get_config!(info);
    let mail_content = MailToLinkTemplate {
        site_name: site_info.name.clone(),
        site_link: site_info.link.clone(),
        user: user.name.clone(),
        link: sig,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| {
        error!("failed to generate mail content, {:?}", x);
        MailInternalError::Render
    })?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "email_check").unwrap().as_str(), 
        &user.name,
        &email){
        Err(MailQueueError::MailBusy) => return Err(
            MailInternalError::Busy.into()
        ),
        Err(MailQueueError::ParseError) => return Err(
            MailInternalError::Parse.into()
        ),
        Ok(_) => {}
    };
    Ok(())
}

pub async fn send_verify_email(user: &User, action: &str, lang: &LanguageIdentifier) -> AppResult<()>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(FeatureNotEnabled.into());
    }

    let code_chars: Vec<char> = (0..6)
    .map(|_| rand::thread_rng().gen_range('0'..'9'))
    .collect();
    let code: String = code_chars.into_iter().collect();
    _ = verificationDao::create(get_db_pool(), user.id, &user.email, &code, 0).await?;
    let site_info = get_config!(info);
    let mail_content = MailVerifyTemplate {
        site_name: site_info.name.clone(),
        site_link: site_info.link.clone(),
        user: user.name.clone(),
        code,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| {
        error!("failed to generate mail content, {:?}", x);
        MailInternalError::Render
    })?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "identity_check").unwrap().as_str(), 
        &user.name,
        &user.email){
            Err(MailQueueError::MailBusy) => return Err(
                MailInternalError::Busy.into()
            ),
            Err(MailQueueError::ParseError) => return Err(
                MailInternalError::Parse.into()
            ),
        Ok(_) => {}
    };
    Ok(())
}