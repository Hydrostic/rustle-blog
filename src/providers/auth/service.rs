// TODO: permission_check marco

use dashmap::DashMap;
use lazy_static::lazy_static;
use lru::LruCache;
use crate::{db::{get_db_pool, rbac as rbacDao}, utils::error_handling::{AppResult, NormalErrorGlobal}};
use once_cell::sync::OnceCell;
lazy_static!{
    pub static ref PERMISSION_CACHE: DashMap<String, Vec<i32>> = DashMap::new();
}
pub static ROLE_CACHE: OnceCell<LruCache<i32, String>> = OnceCell::new(); 
pub async fn check_permission_api(user_id: Option<i32>, permission: &str) -> AppResult<()> {
    let user_roles = match user_id {
        None => {
            vec![0]
        },
        Some(id) => {
            rbacDao::select_user_role(get_db_pool(), id).await?
        }
    };
    if user_roles.len() == 0{
        return NormalErrorGlobal::PermissionDenied.into();
    }
    let roles = PERMISSION_CACHE.get(permission).ok_or(NormalErrorGlobal::PermissionDenied)?;
    for role_id in user_roles{
        if roles.binary_search(&role_id).is_err() {
            return NormalErrorGlobal::PermissionDenied.into();
        }
    }
    Ok(())
}

// function init permission cache
pub async fn init_rbac_cache() -> Result<(), anyhow::Error> {
    let permission_to_roles = rbacDao::select_permission_to_roles(get_db_pool()).await?;
    for entry in permission_to_roles {
        let _ = &PERMISSION_CACHE.entry(entry.permission).or_insert(vec![]).
        push(entry.role);
    }
    // sort role vec for binary search
    PERMISSION_CACHE.iter_mut().for_each(| mut roles | {
        roles.sort();
    });
    Ok(())
}