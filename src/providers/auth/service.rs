// TODO: permission_check marco

use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref PERMISSION_CACHE: DashMap<String, Vec<i32>> = DashMap::new();
}
// pub async fn check_permission_api(depot: &mut Depot, permission: &str) -> AppResult {
//     let user_roles = match depot.session().unwrap().get::<i32>("user_id"){
//         None => {
//             vec![0]
//         },
//         Some(user_id) => {
//             rbacDao::select_user_role_asc_by_role(get_db_pool(), user_id).await?
//         }
//     };
//     if user_roles.len() == 0{
//         return Err(NormalResponseGlobal::PermissionDenied);
//     }
//     let roles = PERMISSION_CACHE.get(permission).unwrap();
//     if roles.binary_search(&role_id).is_err() {
//         return Err(anyhow::anyhow!("permission denied"));
//     }
//     Ok(())
// }

// function init permission cache
pub async fn init_rbac_cache() -> Result<(), anyhow::Error> {
    // let permission_to_roles = rbacDao::select_permission_to_roles(get_db_pool()).await?;
    // for entry in permission_to_roles {
    //     &PERMISSION_CACHE.entry(entry.permission).or_insert(vec![]).
    //     push(entry.role);
    // }
    // // sort role vec for binary search
    // PERMISSION_CACHE.iter_mut().for_each(| mut roles | {
    //     roles.sort();
    // });
    Ok(())
}