use serde::{Serialize,Deserialize};

// use sqlx::MySqlPool;


#[derive(Serialize,Deserialize,Debug)]
pub struct RoleToUser{
    pub id: i32,
    pub role: i32,
    pub user: i32,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct PermissionToRole{
    pub id: i32,
    pub role: i32,
    pub permission: String,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct Role{
    pub id: i32,
    pub name: String,
    pub alias: String,
    pub role_type: i32,
}


// pub async fn select_permission_to_roles(
//     pool: &MySqlPool,
// ) -> AppResult<Vec<PermissionToRole>> {
//     sqlx::query("SELECT * FROM permission_to_roles")
//         .fetch_optional(pool)
//         .await
//         .map_err(|e| {
//             tracing::error!("{:?}", e);
//             AppError::UnexpectedError(StatusCode::INTERNAL_SERVER_ERROR, String::from("db_err"))
//         })
// }

// pub async fn select_user_role_asc_by_role(
//     rb: &dyn Executor,
//     user: i32
// ) -> AppResult<Vec<i32>> {
//     impled!()
// }