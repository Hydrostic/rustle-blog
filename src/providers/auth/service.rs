use dashmap::DashMap;
use once_cell::sync::Lazy;
use tracing::error;
use crate::db::{get_db_pool, rbac as rbacDao};
use crate::types::err::{AppResult, EmptyErrResult};
use crate::types::err::GlobalUserError::PermissionDenied;
use crate::types::service::AppService;

pub static PERMISSION_CACHE: Lazy<DashMap<String, Vec<i32>>> = Lazy::new(|| DashMap::new());

// pub static ROLE_CACHE: OnceCell<LruCache<i32, String>> = OnceCell::new(); 
pub async fn check_permission_api(user_id: Option<i32>, permission: &str) -> AppResult<()> {
    let user_roles = match user_id {
        None => {
            vec![0]
        },
        Some(id) => {
            rbacDao::role_str_to_vec(rbacDao::select_user_role(get_db_pool(), id).await?)?
        }
    };
    if user_roles.len() == 0{
        return Err(PermissionDenied.into());
    }
    let roles = PERMISSION_CACHE.get(permission)
    .ok_or(PermissionDenied)?;
    let mut flag = false;
    for role_id in user_roles{
        if roles.binary_search(&role_id).is_ok() {
            flag = true;
            break;
        }
    }
    if !flag {
        return Err(PermissionDenied.into());
    }
    Ok(())
}

pub fn delete_role_cache(role: i32){
    PERMISSION_CACHE.iter_mut().for_each(|mut arr| {
        arr.retain(|&r| r != role)
    })
}
pub struct RBACService;
impl AppService for RBACService{

    async fn initialize() -> EmptyErrResult<()> {
        let permission_to_roles = rbacDao::select_permission_to_roles(get_db_pool()).await.map_err(|e|{
            error!("failed to get permission-role-relation cache: {}", e);
            ()
        })?;
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
    fn name() -> &'static str {
        "RBACService"
    }
}