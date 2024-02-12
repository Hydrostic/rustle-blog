use serde::Serialize;
use sqlx::MySqlPool;

use super::DBResult;
use tracing::instrument;
use sqlx::FromRow;

#[derive(Serialize,Debug,FromRow)]
pub struct RoleToUser{
    pub id: i32,
    pub role: i32,
    pub user: i32,
}
#[derive(Serialize,Debug,FromRow)]
pub struct PermissionToRole{
    pub id: i32,
    pub role: i32,
    pub permission: String,
}
#[derive(Serialize,Debug,FromRow)]
pub struct Role{
    pub id: i32,
    pub name: String,
    pub alias: String,
    pub role_type: i32,
}
#[derive(Serialize,Debug,FromRow)]
pub struct RoleSimple{
    pub id: i32,
    pub name: String,
}
#[derive(Serialize,Debug,FromRow)]
pub struct UserRoleSimple{
    pub id: i32,
    pub name: String,
    pub user: i32
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
    let res: Vec<(i32,)> = sqlx::query_as("SELECT role FROM role_to_users WHERE user = ? ORDER BY role ASC")
        .bind(user)
        .fetch_all(pool)
        .await?;
    Ok(res.into_iter().map(|t| t.0).collect())
}

#[instrument(err,skip_all)]
pub async fn select_user_role_info(
    pool: &MySqlPool,
    user: Vec<i32>
) -> DBResult<Vec<UserRoleSimple>> {
    let res: Vec<UserRoleSimple> = sqlx::query_as(r#"SELECT role_to_users.user,role_to_users.role as id,IFNULL(roles.name,"") as name FROM role_to_users LEFT JOIN roles ON role_to_users.role = roles.id WHERE role_to_users.user IN (?)"#)
        .bind(user.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(","))
        .fetch_all(pool)
        .await?;
    Ok(res)
}