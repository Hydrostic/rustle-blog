use serde::{Serialize,Deserialize};
use sqlx::MySqlPool;

use super::DBResult;
use tracing::instrument;
use sqlx::FromRow;

#[derive(Serialize,Deserialize,Debug,FromRow)]
pub struct RoleToUser{
    pub id: i32,
    pub role: i32,
    pub user: i32,
}
#[derive(Serialize,Deserialize,Debug,FromRow)]
pub struct PermissionToRole{
    pub id: i32,
    pub role: i32,
    pub permission: String,
}
#[derive(Serialize,Deserialize,Debug,FromRow)]
pub struct Role{
    pub id: i32,
    pub name: String,
    pub alias: String,
    pub role_type: i32,
}

#[instrument(err,skip_all)]
pub async fn select_permission_to_roles(
    pool: &MySqlPool,
) -> DBResult<Vec<PermissionToRole>> {
    sqlx::query_as::<_, PermissionToRole>("SELECT * FROM permission_to_roles")
        .fetch_all(pool)
        .await
}
#[instrument(err,skip_all)]
pub async fn select_user_role(
    pool: &MySqlPool,
    user: i32
) -> DBResult<Vec<i32>> {
    let res: Vec<(i32,)> = sqlx::query_as("SELECT role FROM user_to_roles WHERE user = ? ORDER BY role ASC")
        .bind(user)
        .fetch_all(pool)
        .await?;
    Ok(res.into_iter().map(|t| t.0).collect())
}