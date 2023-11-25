#![allow(non_upper_case_globals)]

use std::sync::RwLock;
use crate::core::config::BaseConfig;
use argon2::{
    password_hash::errors::Error::B64Encoding,
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use lazy_static::lazy_static;
use tracing::info;

lazy_static! {
    pub static ref Ag: Argon2<'static> = Argon2::default();
    pub static ref SALT: RwLock<SaltString> = RwLock::new(SaltString::generate(&mut OsRng));
}

pub fn init_config(c: &mut BaseConfig) -> Result<bool, anyhow::Error> {
    if c.security.password_salt.is_empty() {
        info!("password salt was empty, going to generate one");
        c.security.password_salt = SALT.read().unwrap().as_str().to_string();
        return Ok(true);
    }
    match SaltString::from_b64(&c.security.password_salt) {
        Ok(t) => *SALT.write().unwrap() = t,
        Err(_e @ B64Encoding(e1)) => {
            let e1: anyhow::Error = e1.into();
            return Err(e1.context("salt in config is invalid, {:?}"));
        }
        Err(e) => return Err(e.into()),
    };
    Ok(false)
}
pub fn compare_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed_hash) => Ag
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn generate_password(password: &str) -> Result<String, anyhow::Error> {
    Ok(Ag
        .hash_password(password.as_bytes(), SALT.read().unwrap().as_salt())?
        .to_string())
}
