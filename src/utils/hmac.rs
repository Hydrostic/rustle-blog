use base64::{Engine as _, engine::general_purpose};
use rand::distributions::{Alphanumeric, DistString};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use tracing::info;
use crate::core::config::BaseConfig;
pub fn init_config(c: &mut BaseConfig) -> Result<bool, anyhow::Error> {
    if c.security.credential_secret.is_empty() {
        info!("credential secret was empty, going to generate one");
        c.security.credential_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        return Ok(true);
    }
    Ok(false)
}
pub fn hmac_signature(key: &str, msg: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    mac.update(&msg.as_bytes());

    let code_bytes = mac.finalize().into_bytes();

    return general_purpose::STANDARD.encode(&code_bytes.to_vec());
}