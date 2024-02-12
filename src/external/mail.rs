use crate::get_config;
use crate::types::config::{BaseConfig, ConfigInitializer, MailEnum};
use crate::types::err::EmptyErrResult;
use crate::types::service::AppService;
use fluent_templates::FluentLoader;
use handlebars::{Handlebars, RenderError};
use lettre::{message::Mailbox, Message, SmtpTransport};
use lettre::{
    message::{header::ContentType, MessageBuilder},
    Transport,
};
use lettre::transport::smtp::authentication::Credentials;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info};
pub static MAILER_ENABLED: AtomicBool = AtomicBool::new(false);
use std::sync::OnceLock;

use crate::external::fs::embed::{MailTemplates, LOCALES};

static MAIL_HBS: Lazy<Handlebars> = Lazy::new(|| {
    let mut hbs = Handlebars::new();
    hbs.register_helper("fluent", Box::new(FluentLoader::new(&*LOCALES)));
    hbs.register_embed_templates::<MailTemplates>().unwrap();
    hbs
});
pub struct MailConfig;
impl ConfigInitializer for MailConfig{
    fn initialize(c: &mut BaseConfig) -> Result<bool, ()> {
        match &c.mail {
            MailEnum::Enable(config) => {
                let _: Mailbox = config.from.parse().map_err(|_| {
                    error!("config mail.from not valid, you can either use NAME <MAIL> or MAIL");
                })?;
                SmtpTransport::from_url(&format!("smtps://{}:{}", config.host, config.port)).map_err(|_| {
                    error!("smtp address invalid");
                })?;
                
            },
            MailEnum::Disabled => {
                info!("mail function disabled");
            }
        };
        Ok(false)
    }
}


#[derive(Serialize)]
pub struct MailToLinkTemplate {
    pub site_name: String,
    pub site_link: String,
    pub user: String,
    pub link: String,
    pub action: String,
    pub lang: String,
}
impl MailToLinkTemplate {
    pub fn generate(&self) -> Result<String, RenderError> {
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
    pub lang: String,
}
impl MailVerifyTemplate {
    pub fn generate(&self) -> Result<String, RenderError> {
        MAIL_HBS.render("verify.hbs", &serde_json::json!(self))
    }
}

pub static TX: OnceLock<Sender<(MessageBuilder, String)>> = OnceLock::new();
pub struct MailService;
impl AppService for MailService {
    fn name() -> &'static str {
        "MailService"
    }
    async fn initialize() -> EmptyErrResult<()> {
            let config = get_config!(mail);
            let config = match config{
                MailEnum::Enable(c) => c,
                MailEnum::Disabled => return Ok(())
            };
            let from: Mailbox = config.from.parse().unwrap(); // checked before
            let creds = Credentials::new(config.smtp_user_name.clone(), config.smtp_password.clone());
                let transporter =
                    SmtpTransport::from_url(&format!("smtps://{}:{}", config.host, config.port)).unwrap()
                    .credentials(creds)
                    .build();
                if let Err(e) = transporter.test_connection() {
                    error!("mail connection not available, continue anyway: {:?}", e);
                }
                MAILER_ENABLED.store(true, atomic::Ordering::SeqCst);
                let (tx, mut rx) = mpsc::channel::<(MessageBuilder, String)>(config.max_queue_capacity as usize);
                TX.get_or_init(|| tx);
                tokio::spawn(async move {
                    while let Some(mail) = rx.recv().await {
                        let email = transporter.send(
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
                Ok(())
    }
}

pub enum MailQueueError {
    MailBusy,
    ParseError,
}
pub fn try_send(
    content: String,
    subject: &str,
    object: &str,
    email: &str,
) -> Result<(), MailQueueError> {
    if let Err(_) = TX.get().clone().unwrap().try_send((
        Message::builder()
            .to(format!("{} <{}>", object, email).parse().map_err(|e| {
                error!("mail<to> not valid, possible of program error, {:?}", e);
                MailQueueError::ParseError
            })?)
            .subject(subject),
        content,
    )) {
        return Err(MailQueueError::MailBusy);
    }
    Ok(())
}
