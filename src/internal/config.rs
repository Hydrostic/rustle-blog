use std::fs;
use once_cell::sync::Lazy;
use std::sync::Arc;
use sync_cow::SyncCow;
use tracing::error;
use crate::external::mail::MailConfig;
use crate::types::config::BaseConfig;
use crate::types::err::EmptyErrResult;
use crate::types::service::AppService;
use crate::utils::hmac::CredentialConfig;
use crate::utils::paseto::AuthTokenConfig;
use crate::utils::password_salt::PasswordSaltConfig;

pub static SETTINGS: Lazy<Arc<SyncCow<BaseConfig>>> = Lazy::new(|| {
    Arc::new(SyncCow::new(BaseConfig{..Default::default()}))
});
macro_rules! initialize_config {
    (&mut $config:expr, $($step:ident),*) => {{
        use crate::types::config::ConfigInitializer;
        let mut need_rewrite = false;
        let mut has_err = false;
        $(
            if let Ok(b) = $step::initialize(&mut $config){
                need_rewrite = need_rewrite || b;
            }else{
                has_err = true;
            }
        )*
        if has_err{
            Err(())
        }else{
            Ok(need_rewrite)
        }
    }};
}
pub struct ConfigService;
impl AppService for ConfigService{
    async fn initialize() -> EmptyErrResult<()> {
        load_config()
    }

    fn name() -> &'static str {
        "ConfigService"
    }
}
pub fn load_config() -> EmptyErrResult<()>{

    let config_content = fs::read("./config.toml")
        .map_err(|e| error!("failed to open config file, {:?}", e))?;
    let config_content = String::from_utf8(config_content)
        .map_err(|e| error!("failed to open config file, {:?}", e))?;
    let mut config: BaseConfig = toml::from_str(&config_content)
        .map_err(|e| error!("corrupted config, {:?}", e))?;
    match initialize_config!(
        &mut config,
        PasswordSaltConfig,
        AuthTokenConfig,
        CredentialConfig,
        MailConfig
    ){
        Ok(true) => {
            // config need rewrite
            let new_content = toml::to_string(&config).map_err(|e| error!("deserialization failure, {:?}", e))?;
            fs::write("./config.toml", new_content).map_err(|e| error!("failed to write config file, {:?}", e))?;
        },
        Ok(false) => {},
        Err(_) => {
            return Err(());
        }
    };
    SETTINGS.edit(|c| {
        *c = config;
    });
    Ok(())
}
#[macro_export]
macro_rules! get_config {
    () => {
        (crate::internal::config::SETTINGS.read())
    };
    ($path:ident) => {
        (&(get_config!().$path))
    }
}