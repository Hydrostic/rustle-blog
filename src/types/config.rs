use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DatabaseConfig {
    #[serde_inline_default(String::from("rustle_blog"))]
    pub db_name: String,
    #[serde_inline_default(String::from("rustle"))]
    pub user_name: String,
    pub password: String,
    #[serde_inline_default(String::from("127.0.0.1"))]
    pub host: String,
    #[serde_inline_default(3306)]
    pub port: u32,
}
#[serde_inline_default]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct MailConfig {
    pub from: String,
    pub smtp_user_name: String,
    pub smtp_password: String,
    pub host: String,
    #[serde_inline_default(465)]
    pub port: u32,
    #[serde_inline_default(100)]
    pub max_queue_capacity: u32,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum MailEnum {
    Enable(MailConfig),
    #[default] Disabled,
}
#[serde_inline_default]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct HttpConfig {
    #[serde_inline_default(String::from("127.0.0.1"))]
    pub host: String,
    #[serde_inline_default(5800)]
    pub port: u16,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SecurityConfig {
    pub password_salt: String,
    pub auth_token_secret: String,
    pub credential_secret: String,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct InfoConfig {
    pub name: String,
    pub link: String,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BaseConfig {
    pub database: DatabaseConfig,
    pub http: HttpConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    pub mail: MailEnum,
    pub info: InfoConfig,
}
pub trait ConfigInitializer {
    fn initialize(c: &mut BaseConfig) -> Result<bool, ()>;
}