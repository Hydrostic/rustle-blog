use rustle_derive::ErrorHelper;
use serde::Serialize;
use sqlx::{Executor, MySql, MySqlPool, QueryBuilder, Row};

use crate::types::{err::{AppResult, GlobalUserError}, join::Joinable};

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


// design
// pub struct xx{
//  #[foreign_key]    
// }
// #[derive(Serialize,Debug,FromRow,Clone)]
// pub struct UserRoleSimple{
//     pub user: i32,
//     pub roles: Vec<RoleSimple>,
// }
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
    // todo: improve
    tx.commit().await?;
    Ok(Ok(()))
}

#[instrument(err,skip_all)]
pub async fn add_role(
    pool: &MySqlPool,
    name: &str,
    alias: &str,
    permissions: Vec<&str>
) -> DBResult<AppResult<()>> {
    let mut tx = pool.begin().await?;
    let role_id = tx.execute(sqlx::query(r#"
        INSERT INTO roles (name,alias,role_type) VALUES (?,?,0)
    "#).bind(name).bind(alias)
    ).await?.last_insert_id();
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        // Note the trailing space
        "INSERT INTO permission_to_roles(role, permission) "
    );
    
    query_builder.push_values(permissions, |mut b, entry| {
        b.push_bind(role_id).push_bind(entry);
    });
    tx.execute(query_builder.build()).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

#[instrument(err,skip_all)]
pub async fn delete_role_permission(
    pool: &MySqlPool,
    role: i32,
    permission: &str
) -> DBResult<()> {
    sqlx::query("DELETE roles WHERE role = ? AND permission = ?")
        .bind(role)
        .bind(permission)
        .execute(pool)
        .await?;
    Ok(())
}
#[instrument(err,skip_all)]
pub async fn add_role_permission(
    pool: &MySqlPool,
    role: i32,
    permission: &str
) -> DBResult<()> {
    sqlx::query("INSERT INTO roles (role, permission) VALUES (?,?)")
        .bind(role)
        .bind(permission)
        .execute(pool)
        .await?;
    Ok(())
}
#[instrument(err,skip_all)]
pub async fn join_user_role_info(
    pool: &MySqlPool,
    outer: &mut Vec<impl Joinable<Option<Vec<i32>>, Vec<RoleSimple>>>
) -> DBResult<()>{
    let mut role_ids: Vec<i32> = Vec::with_capacity(outer.len());
    for entry in &mut *outer{
        if let Some(t) = entry.get_ref().0{
            role_ids.extend(t);
        }
    }
    let role_infos: Vec<RoleSimple> = sqlx::query_as(&format!("SELECT id,name FROM roles WHERE id in ({})",
        role_ids.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(",")
    )).fetch_all(pool).await?;
    for entry in &mut *outer {
        let entry = entry.get_ref();
        let key = match entry.0{
            None => continue,
            Some(t) => t
        };
        entry.1.extend(
            role_infos.iter().filter_map(|role| {
                if key.contains(&role.id){
                    Some(role.clone())
                }else{
                    None
                }
            })
        );
    }
    Ok(())
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
    if roles.is_empty() {
        Ok(vec![])
    }else{
        roles.split(",").map(|t| t.parse()).collect::<Result<Vec<i32>, _>>()
        .map_err(|_| 
            RBACUserError::RoleStrCorrupted.into()
        )
    }
}