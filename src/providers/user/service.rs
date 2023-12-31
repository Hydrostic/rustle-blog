use fluent_templates::{Loader, LanguageIdentifier};
use rand::Rng;

use crate::core::config::read_config;
use crate::communication::mail::{self, MailToLinkTemplate, MailVerifyTemplate, MAILER_ENABLED, MailQueueError};
use crate::db::user::User;
use crate::db::{verification as verificationDao, RB};
use crate::fs::embed::LOCALES;
use crate::utils::hmac::hmac_signature;
use std::sync::atomic;

pub enum SendMailError{
    FeatureNotEnabled,
    MailBusy,
    Unexpected(anyhow::Error)
}
pub async fn send_tolink_email(email: &str, user: &User, action: &str, lang: &LanguageIdentifier) -> Result<(), SendMailError>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(SendMailError::FeatureNotEnabled);
    }
    let verification_res = verificationDao::create(&mut RB.clone(), user.id, &user.email, "", 0).await.map_err(|x| SendMailError::Unexpected(x.into()))?;
    let mut sig = hmac_signature(
        &read_config().security.credential_secret,
        &verification_res.last_insert_id.to_string(),
    );
    sig.push('.');
    sig.push_str(&verification_res.last_insert_id.to_string());
    let mail_content = MailToLinkTemplate {
        site_name: read_config().info.name.clone(),
        site_link: read_config().info.link.clone(),
        user: user.name.clone(),
        link: sig,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| SendMailError::Unexpected(x.into()))?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "email_check").unwrap().as_str(), 
        &user.name,
        &email){
        Err(MailQueueError::MailBusy) => return Err(SendMailError::MailBusy),
        Err(MailQueueError::ParseError(e)) => return Err(SendMailError::Unexpected(e.into())),
        Ok(_) => {}
    };
    Ok(())
}

pub async fn send_verify_email(user: &User, action: &str, lang: &LanguageIdentifier) -> Result<(), SendMailError>{
    if !MAILER_ENABLED.load(atomic::Ordering::Relaxed) {
        return Err(SendMailError::FeatureNotEnabled);
    }

    let code_chars: Vec<char> = (0..6)
    .map(|_| rand::thread_rng().gen_range('0'..'9'))
    .collect();
    let code: String = code_chars.into_iter().collect();
    _ = verificationDao::create(&mut RB.clone(), user.id, &user.email, &code, 0).await.map_err(|x| SendMailError::Unexpected(x.into()))?;
    let mail_content = MailVerifyTemplate {
        site_name: read_config().info.name.clone(),
        site_link: read_config().info.link.clone(),
        user: user.name.clone(),
        code,
        action: LOCALES.lookup(lang, action).unwrap(),
        lang: lang.to_string(),
    };
    let mail_str = mail_content.generate().map_err(|x| SendMailError::Unexpected(x.into()))?;
     match mail::try_send(mail_str, 
        LOCALES.lookup(lang, "identity_check").unwrap().as_str(), 
        &user.name,
        &user.email){
        Err(MailQueueError::MailBusy) => return Err(SendMailError::MailBusy),
        Err(MailQueueError::ParseError(e)) => return Err(SendMailError::Unexpected(e.into())),
        Ok(_) => {}
    };
    Ok(())
}