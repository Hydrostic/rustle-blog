use base64::{Engine as _, engine::general_purpose};
use rand::distributions::{Alphanumeric, DistString};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use tracing::info;
use crate::types::config::{BaseConfig, ConfigInitializer};
pub struct CredentialConfig;
impl ConfigInitializer for CredentialConfig{
    fn initialize(c: &mut BaseConfig) -> Result<bool, ()> {
        if c.security.credential_secret.is_empty() || c.security.credential_secret.as_bytes().len() < 32 {
            info!("credential secret was empty / length smaller than 32, going to generate one");
            c.security.credential_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
            return Ok(true);
        }
        Ok(false)
    }
}
pub fn hmac_signature(key: &str, msg: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    mac.update(&msg.as_bytes());

    let code_bytes = mac.finalize().into_bytes();

    return general_purpose::STANDARD.encode(&code_bytes.to_vec());
}

pub fn hmac_verify(key: &str, msg: &str, received: &str) -> bool{
    hmac_signature(key, msg) == received
}