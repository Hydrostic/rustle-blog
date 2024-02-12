use crate::types::config::BaseConfig;
use tracing::{info, error};
use crate::types::config::ConfigInitializer;
use argon2::{
    password_hash::SaltString,
    password_hash::rand_core::OsRng,
    password_hash::PasswordHash,
    password_hash::PasswordHasher,
    password_hash::PasswordVerifier,
    password_hash,
    Argon2
};
use rustle_derive::ErrorHelper;
use crate::types::err::AppResult;

pub struct PasswordSaltConfig;
impl ConfigInitializer for PasswordSaltConfig{
    fn initialize(c: &mut BaseConfig) -> Result<bool, ()> {
        let len = c.security.password_salt.len();

        if len == 0{
            info!("generate a new password salt");
            c.security.password_salt = SaltString::generate(&mut OsRng).to_string();
            return Ok(true);
        }
        match SaltString::from_b64(&c.security.password_salt){
            Ok(_) => {},
            Err(e) => {
                error!("failed to parse password salt: {:?}", e);
                return Err(());
            }
        }
        Ok(false)
    }
}

pub fn compare_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed_hash) =>
            Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}
#[derive(ErrorHelper)]
#[err(msg = "error.password.salt", internal)]
struct PasswordSaltInternalError;

#[derive(ErrorHelper)]
#[err(user)]
struct PasswordTooLong;
pub fn generate_password(password: &str, salt: &str) -> AppResult<String> {
    let salt = SaltString::from_b64(salt).unwrap();
    match Argon2::default()
        .hash_password(password.as_bytes(), &salt){
            Ok(t) => Ok(t.to_string()),
            Err(password_hash::Error::OutputSize { .. }) => Err(PasswordTooLong.into()),
            Err(e) => {
                error!("failed to salt for password: {:?}", e);
                Err(PasswordSaltInternalError.into())
            }
        }
}
