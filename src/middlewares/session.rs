use crate::core::config::BaseConfig;
use lazy_static::lazy_static;
use rand::distributions::{Alphanumeric, DistString};
use std::sync::RwLock;
use tracing::info;
use validator::HasLen;
use salvo::session::{SessionHandler, CookieStore};

lazy_static! {
    pub static ref SECRET: RwLock<String> = RwLock::new("".to_string());
}

pub fn init_config(c: &mut BaseConfig) -> Result<bool, anyhow::Error> {
    if c.security.session_secret.is_empty() || c.security.session_secret.as_bytes().length() < 64 {
        info!("session secret was empty or length less than 64, going to generate one");
        c.security.session_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
        *SECRET.write().unwrap() = c.security.session_secret.clone();
        return Ok(true);
    }
    *SECRET.write().unwrap() = c.security.session_secret.clone();
    Ok(false)
}

pub fn new_middleware() -> SessionHandler<CookieStore>{
    SessionHandler::builder(
        CookieStore::new(),
        (*SECRET.read().unwrap()).as_bytes(),
    )
    .build()
    .unwrap()
}