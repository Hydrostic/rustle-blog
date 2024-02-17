use std::collections::HashMap;

use rustle_derive::ErrorHelper;
use serde::Serialize;
use sqlx::{Executor, MySqlPool, Row};

use crate::types::err::{AppResult, GlobalUserError};

use super::DBResult;
use tracing::instrument;
use sqlx::FromRow;


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
    pub role_type: i8,
}
#[derive(Serialize,Debug,FromRow,Clone)]
pub struct RoleSimple{
    pub id: i32,
    pub name: String,
}
#[derive(Serialize,Debug,FromRow,Clone)]
pub struct UserRoleSimple{
    pub user: i32,
    pub roles: Vec<RoleSimple>,
}
pub const DEFAULT_ROLE_STR: &'static str = "0,1";
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
) -> DBResult<String> {
    let res: (String,) = match sqlx::query_as("SELECT roles FROM users WHERE id = ? LIMIT 1")
        .bind(user)
        .fetch_one(pool)
        .await{
            Ok(t) => t,
            Err(sqlx::Error::RowNotFound) => return Ok(String::from("")),
            Err(e) => return Err(e)
        };
    Ok(res.0)
}
#[instrument(err,skip_all)]
pub async fn update_user_roles(
    pool: &MySqlPool,
    user: i32,
    roles: &str
) -> DBResult<()> {
    sqlx::query("UPDATE users SET roles = ? WHERE id = ?")
        .bind(roles)
        .bind(user)
        .execute(pool)
        .await?;
    Ok(())
}
#[instrument(err,skip_all)]
pub async fn list_roles(
    pool: &MySqlPool,
    limit: i32,
    offset: i32
) -> DBResult<(i32, Vec<Role>)> {
    let quantity: i32 = sqlx::query_as::<_, (i32, )>("SELECT COUNT(*) FROM roles")
        .fetch_one(pool)
        .await?.0;
    let roles = sqlx::query_as(&format!("SELECT * FROM roles LIMIT {offset},{limit}"))
    .fetch_all(pool)
    .await?;
    Ok((quantity, roles))
}
#[instrument(err,skip_all)]
pub async fn delete_role(
    pool: &MySqlPool,
    role: i32,
) -> DBResult<AppResult<()>> {
    let mut tx = pool.begin().await?;
    let role_type = tx.fetch_optional(sqlx::query(r#"
    SELECT role_type FROM roles WHERE id = ? LIMIT 1 
"#)
    .bind(role)).await?;
    match role_type {
        None => {
            return Ok(Err(GlobalUserError::NotFound.into()));
        },
        Some(t) => {
            if t.try_get::<i8, _>("role_type")? == 1i8 {
                return Ok(Err(GlobalUserError::SystemReserved.into()));
            }
        }
    }
    tx.execute(sqlx::query(r#"
        DELETE FROM permission_to_roles WHERE role = ?;
        "#).bind(role)).await?;
        tx.execute(sqlx::query(r#"
        DELETE FROM roles WHERE id = ?;
        "#).bind(role)).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

use futures_util::stream::TryStreamExt;
#[instrument(err,skip_all)]
pub async fn select_user_role_info(
    pool: &MySqlPool,
    user: Vec<i32>
) -> DBResult<Vec<UserRoleSimple>> {
    let user_roles: Vec<(String, i32)> = sqlx::query_as(r#"
        SELECT roles,id FROM users WHERE id in (?)
    "#).bind(user.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(",")).fetch_all(pool).await?;
    let mut role_ids: Vec<i32> = Vec::with_capacity(user_roles.len());
    user_roles.iter().for_each(|(role_str,_)| {
        role_ids.extend_from_slice(&role_str.split(",").map(|t| t.parse()).collect::<Result<Vec<i32>, _>>().unwrap_or(vec![]));
    });
    let mut role_map: HashMap<i32, String> = HashMap::with_capacity(role_ids.len());
    let mut role_info_res = sqlx::query(r#"
        SELECT id,name FROM roles WHERE id in (?)"#)
        .bind(role_ids.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(",")).fetch(pool);
    while let Some(t) = role_info_res.try_next().await?{
        role_map.insert(t.try_get("id")?, t.try_get("name")?);
    }
    Ok(user_roles.into_iter().map(|t| {
        UserRoleSimple{
            user: t.1,
            roles: t.0.split(",").filter_map(|t| {
                let role_id = match t.parse(){
                    Err(_) => return None,
                    Ok(t) => t
                };
                Some(RoleSimple{
                    id: role_id,
                    name: role_map.get(&role_id).unwrap_or(&String::from("")).to_owned()
                })
            }).collect()
        }
    }).collect())
}

#[derive(ErrorHelper)]
#[err(user, default_msg)]
pub enum RBACUserError{
    RoleTooLong,
    RoleStrCorrupted
}
const MAX_ROLE_STR_LEN: usize = 255;
pub fn role_vec_to_str(roles: Vec<i32>) -> AppResult<String>{
    let role_str = roles.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(",");
    if role_str.len() > MAX_ROLE_STR_LEN{
        Err(RBACUserError::RoleTooLong.into())
    } else {
        Ok(role_str)
    }
}
pub fn role_str_to_vec(roles: String) -> AppResult<Vec<i32>>{
    roles.split(",").map(|t| t.parse()).collect::<Result<Vec<i32>, _>>()
        .map_err(|_| RBACUserError::RoleStrCorrupted.into())
}