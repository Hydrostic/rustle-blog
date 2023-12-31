use crate::core::config::BaseConfig;
use anyhow::Context;
use handlebars::{Handlebars, RenderError};
use fluent_templates::FluentLoader;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::Mailbox, SmtpTransport, Message};
use lettre::{
    message::{header::ContentType, MessageBuilder},
    Transport,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info};
pub static MAILER_ENABLED: AtomicBool = AtomicBool::new(false);
use std::sync::OnceLock;
use crate::fs::embed::{MailTemplates, LOCALES};
static MAIL_HBS: Lazy<Handlebars> = Lazy::new(||{
    let mut hbs = Handlebars::new();
    hbs.register_helper("fluent", Box::new(FluentLoader::new(&*LOCALES)));
    hbs.register_embed_templates::<MailTemplates>().unwrap();
    hbs
});


pub fn init_config(c: &mut BaseConfig) -> Result<bool, anyhow::Error> {
    if c.mail.host.is_empty()
        || c.mail.port == 0
        || c.mail.from.is_empty()
        || c.mail.smtp_user_name.is_empty()
        || c.mail.smtp_password.is_empty()
    {
        info!("some items of mail config are empty, just continue with mail disabled");
        c.mail.mail_enabled = false;
    } else {
        let from: Mailbox = c
            .mail
            .from
            .parse()
            .context("config mail.from not valid, you can either use NAME <MAIL> or MAIL")?;
        let creds = Credentials::new(c.mail.smtp_user_name.clone(), c.mail.smtp_password.clone());
        let transporter =
            match SmtpTransport::from_url(&format!("smtps://{}:{}", c.mail.host, c.mail.port)) {
                Ok(t) => t,
                Err(e) => return Err(e.into()),
            }
            .credentials(creds)
            .build();
        if let Err(e) = transporter.test_connection() {
            error!("mail connection not available, continue anyway: {:?}", e);
        }

        init_queue(c.mail.max_queue_capacity, transporter, from);
        c.mail.mail_enabled = true;
        MAILER_ENABLED.store(true, atomic::Ordering::SeqCst);
    }
    
       
    Ok(false)
    
}

#[derive(Serialize)]
pub struct MailToLinkTemplate {
    pub site_name: String,
    pub site_link: String,
    pub user: String,
    pub link: String,
    pub action: String,
    pub lang: String
}
impl MailToLinkTemplate{
    pub fn generate(&self) -> Result<String, RenderError>{
        MAIL_HBS.render("tolink.hbs", &serde_json::json!(self))
    }
}
#[derive(Serialize)]
pub struct MailVerifyTemplate {
    pub site_name: String,
    pub site_link: String,
    pub user: String,
    pub code: String,
    pub action: String,
    pub lang: String
}
impl MailVerifyTemplate{
    pub fn generate(&self) -> Result<String, RenderError>{
        MAIL_HBS.render("verify.hbs", &serde_json::json!(self))
    }
}

pub static TX: OnceLock<Sender<(MessageBuilder, String)>> = OnceLock::new();

fn init_queue(max_capacity: u32, mailer: SmtpTransport, from: Mailbox) {
    let (tx, mut rx) = mpsc::channel::<(MessageBuilder, String)>(max_capacity as usize);
    TX.get_or_init(|| tx);
    tokio::spawn(async move {
        while let Some(mail) = rx.recv().await {
            let email = mailer.send(
                &mail
                    .0
                    .from(from.clone())
                    .header(ContentType::TEXT_HTML)
                    .body(mail.1)
                    .unwrap(),
            );
            if let Err(e) = email {
                error!("mail sending error: {:?}", e);
            }
        }
    });
}

pub enum MailQueueError {
    MailBusy,
    ParseError(anyhow::Error),
}
pub fn try_send(content: String, subject: &str, object: &str, email: &str) -> Result<(),MailQueueError> {
    if let Err(_) = TX.get().clone().unwrap().try_send((
        Message::builder()
            .to(format!("{} <{}>", subject, email)
                .parse()
                .context("mail<to> not valid, possible of program error").map_err(|e| MailQueueError::ParseError(e))?)
            .subject(subject),
        content,
    )) {
        return Err(MailQueueError::MailBusy);
    }
    Ok(())
}