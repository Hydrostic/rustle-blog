use pasetors::keys::{Generate, SymmetricKey};
use pasetors::version4::V4;
use tracing::{error, info};
use crate::types::config::{BaseConfig, ConfigInitializer};
use base64::{Engine as _, engine::general_purpose};
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::Local;
use pasetors::token::UntrustedToken;
use rustle_derive::ErrorHelper;
use crate::types::err::AppResult;
use crate::types::err::GlobalUserError::StatusUnauthorized;

pub struct AuthTokenConfig;
impl ConfigInitializer for AuthTokenConfig {
    fn initialize(c: &mut BaseConfig) -> Result<bool, ()> {
        let len = c.security.auth_token_secret.len();
        if len == 0 {
            info!("generate a new auth token secret");
            c.security.auth_token_secret = match SymmetricKey::<V4>::generate() {
                Err(e) => {
                    error!("random not available, check your operation system: {:?}", e);
                    return Err(());
                },
                Ok(s) => general_purpose::STANDARD.encode(s.as_bytes())
            };
            return Ok(true);
        }
        let key_bytes = general_purpose::STANDARD.decode(&c.security.auth_token_secret).map_err(|_|{
            error!("auth_token_secret is not a valid b64 string");
        })?;
        match SymmetricKey::<V4>::from(&*key_bytes){
            Err(e) => {
                error!("failed to parse paseto key: {:?}", e);
                Err(())
            },
            Ok(_) => Ok(false)
        }
    }
}

#[derive(ErrorHelper)]
#[err(msg = "error.token", internal)]
struct TokenInternalError;
fn generate_token(secret: &str, claims: &Claims, assertion: Option<&str>) -> AppResult<String>{
    pasetors::local::encrypt(
        &SymmetricKey::<V4>::from(&*general_purpose::STANDARD.decode(secret).unwrap()).unwrap(),
        claims,
        None,
        assertion.map(|s| s.as_bytes())
    ).map(|token| token.as_str().to_string()).map_err(|e| {
        error!("failed to generate token: {:?}", e);
        TokenInternalError.into()
    })
}
const ACCESS_TOKEN_EXPIRE_TIME: i64 = 60*5; // 5 minutes
pub fn generate_access_token(secret: &str, user: i32) -> AppResult<String>{
    let mut claims = Claims::new().unwrap();
    claims.issuer("rustle_backend").unwrap();
    claims.audience("rustle_frontend").unwrap();
    claims.subject("auth_access").unwrap();
    claims.add_additional("user", user).unwrap();
    claims.expiration(
        &(chrono::Utc::now() + chrono::Duration::seconds(ACCESS_TOKEN_EXPIRE_TIME)).to_rfc3339()
    ).unwrap();
    claims.issued_at(&chrono::Utc::now().to_rfc3339()).unwrap();
    generate_token(secret, &claims, None)
}

pub fn verify_access_token(secret: &str, token: &str) -> AppResult<i32>{
    let mut validation_rules = ClaimsValidationRules::new();
    validation_rules.validate_issuer_with("rustle_backend");
    validation_rules.validate_audience_with("rustle_frontend");
    validation_rules.validate_subject_with("auth_access");
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token).map_err(|_| StatusUnauthorized)?;
    let trusted_token = pasetors::local::decrypt(
        &SymmetricKey::<V4>::from(&*general_purpose::STANDARD.decode(secret).unwrap()).unwrap(),
        &untrusted_token, &validation_rules, None, None).map_err(|_| StatusUnauthorized)?;
    let claims = trusted_token.payload_claims().ok_or(StatusUnauthorized)?;
    Ok(claims.get_claim("user").ok_or(StatusUnauthorized)?
        .as_i64().ok_or(StatusUnauthorized)? as i32)
}