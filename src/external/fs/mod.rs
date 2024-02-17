

use dashmap::DashMap;
use once_cell::sync::Lazy;
use rustle_derive::ErrorHelper;
use tracing::error;

use crate::types::err::EmptyErrResult;
use crate::types::service::AppService;
use crate::db::{fs as fsDao, get_db_pool};
use self::interface::FsProvider;
use self::local::LocalFs;

pub mod embed;
pub mod local;
pub mod interface;
pub static FS_POLICY_CACHE: Lazy<DashMap<i32, FsPolicy>> = Lazy::new(|| DashMap::new());
pub struct FsService;
pub struct FsPolicy{
    pub info: fsDao::FsPolicy,
    pub instance: FsProvider 
}
impl AppService for FsService {
    fn name() -> &'static str {
        "FsService"
    }
    async fn initialize() -> EmptyErrResult<()> {
        for policy_info in fsDao::get_all_policies(get_db_pool()).await.map_err(|e|{
            error!("failed to get fs policy cache: {}", e);
        })?{
            if policy_info.extension.eq(LocalFs::EXTENSION_NAME){
                let path = match policy_info.meta.get("path"){
                    Some(p) => p.to_string(),
                    None => {
                        error!("path not found in the config of local fs");
                        continue;
                    }
                };
                let _ = FS_POLICY_CACHE.insert(policy_info.id, FsPolicy{
                    instance: FsProvider::LocalProvider(LocalFs::initialize(path).await?),
                    info: policy_info
                });
            } else {
                todo!()
            }
        }
        Ok(())
    }
}

#[derive(ErrorHelper)]
#[err(user, default_msg)]
pub enum FsUserError{
    PolicyNotFound,
    PathNameNotValid
}

pub const DEFAULT_POLICY_ID: i32 = 1;