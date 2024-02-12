use serde::{Serialize,Deserialize};

use sqlx::mysql::MySqlRow;
use sqlx::Row;
use tracing::instrument;
use sqlx::MySqlPool;
use sqlx::FromRow;
use super::DBResult;
#[derive(Deserialize,Serialize,Debug)]
pub struct User{
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: Option<String>,
}
impl<'r> FromRow<'r, MySqlRow> for User{
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self{
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            password: row.try_get("password").unwrap_or(None)
        })
    }
}
#[derive(Deserialize,Serialize,Debug)]
pub struct UserIdName{
    pub id: i32,
    pub name: String,
}
#[instrument(err,skip_all)]
pub async fn select_by_identity_with_password(
    pool: &MySqlPool,
    identity: &str
) -> DBResult<Option<User>> {
    sqlx::query_as::<_,User>("SELECT * FROM users WHERE name = ? OR email = ?")
        .bind(identity)
        .bind(identity)
        .fetch_optional(pool)
        .await
}

#[instrument(err,skip_all)]
pub async fn select_by_id(
    pool: &MySqlPool,
    id: i32
) -> DBResult<Option<User>> {
    sqlx::query_as::<_,User>("SELECT id,name,email FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[instrument(err,skip_all)]
pub async fn select_by_id_with_password(
    pool: &MySqlPool,
    id: i32
) -> DBResult<Option<User>> {
    sqlx::query_as::<_,User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[instrument(err,skip_all)]
pub async fn select_by_email(
    pool: &MySqlPool,
    email: &str
) -> DBResult<Option<User>> {
    sqlx::query_as::<_,User>("SELECT id,name,email FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
}

#[instrument(err,skip_all)]
pub async fn create(
    pool: &MySqlPool,
    name: &str,
    email: &str,
    password: &str
) -> DBResult<i32> {
    Ok(sqlx::query("INSERT INTO users (name,email,password) VALUES (?,?,?)")
        .bind(name)
        .bind(email)
        .bind(password)
        .execute(pool)
        .await?.last_insert_id() as i32)
}

#[instrument(err,skip_all)]
pub async fn update_password(
    pool: &MySqlPool,
    id: i32,
    password: &str
) -> DBResult<()> {
    sqlx::query("UPDATE users SET password = ? WHERE id = ?")
        .bind(password)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[instrument(err,skip_all)]
pub async fn update_email(
    pool: &MySqlPool,
    id: i32,
    email: &str
) -> DBResult<()>{
    sqlx::query("UPDATE users SET email = ? WHERE id = ?")
        .bind(email)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
}

#[instrument(err,skip_all)]
pub async fn get_list(pool: &MySqlPool, limit: i32, offset: i32) -> DBResult<Vec<User>> {
    let res: Vec<User> = sqlx::query_as("SELECT id,email,name FROM users LIMIT ?,?")
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(res)
}