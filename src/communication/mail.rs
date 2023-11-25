use crate::core::config::BaseConfig;
use anyhow::Context;
use askama::Template;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::Mailbox, SmtpTransport};
use lettre::{Message, Transport};
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info};
pub static MAILER_ENABLED: AtomicBool = AtomicBool::new(false);
use std::sync::OnceLock;
// pub static MAILER: RwLock<Option<SmtpTransport>> = RwLock::new(None);
// pub static MAIL_FROM: RwLock<Option<Mailbox>> = RwLock::new(None);
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

#[derive(Template)]
#[template(path = "email/tolink.html")]

pub struct MailToLinkTemplate {
    pub site_name: String,
    pub greeting: String,
    pub probably_action: String,
    pub warning: String,
    pub link: String,
    pub action: String,
    pub to: String,
    pub subject: String,
}
impl MailTemplate for MailToLinkTemplate {
    fn get_content(&self) -> String {
        self.to_string()
    }
    fn get_to(&self) -> String {
        self.to.to_owned()
    }
    fn get_subject(&self) -> String {
        self.subject.to_owned()
    }
}
pub trait MailTemplate: Send + Sync {
    fn get_content(&self) -> String;
    fn get_to(&self) -> String;
    fn get_subject(&self) -> String;
}
pub static TX: OnceLock<Sender<Box<dyn MailTemplate>>> = OnceLock::new();

fn init_queue(max_capacity: u32, mailer: SmtpTransport, from: Mailbox) {
    let (tx, mut rx) = mpsc::channel::<Box<dyn MailTemplate>>(max_capacity as usize);
    TX.get_or_init(|| tx);
    tokio::spawn(async move {
        while let Some(mail) = rx.recv().await {
            let to_mailbox = match mail.get_to().parse() {
                Ok(m) => m,
                Err(e) => {
                    error!("mail<to> not valid, possible of program error: {:?}", e);
                    continue;
                }
            };
            let email = mailer.send(
                &Message::builder()
                    .to(to_mailbox)
                    .from(from.clone())
                    .subject(mail.get_subject())
                    .header(ContentType::TEXT_HTML)
                    .body(mail.get_content())
                    .unwrap(),
            );
            if let Err(e) = email {
                error!("mail sending error: {:?}", e);
            }
        }
    });
}
