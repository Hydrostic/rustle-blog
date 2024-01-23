use serde::Serialize;
use sqlx::{MySqlPool, types::chrono};
use tracing::instrument;
use super::DBResult;
use sqlx::FromRow;
#[derive(Serialize,Debug,FromRow)]
pub struct Verification{
    pub id: i32,
    pub user: i32,
    pub identity: String,
    pub random_code: String,
    pub action: i16,
    pub created_at: chrono::NaiveDateTime
}
#[instrument(err,skip_all)]
pub async fn create(
    pool: &MySqlPool,
    user: i32,
    identity: &str,
    random_code: &str,
    action: i16,
) -> DBResult<i32> {
    Ok(sqlx::query("INSERT INTO verifications (user,identity,random_code,action,created_at) VALUES (?,?,?,?,NOW())")
        .bind(user)
        .bind(identity)
        .bind(random_code)
        .bind(action)
        .execute(pool)
        .await?.last_insert_id() as i32)
}
#[instrument(err,skip_all)]
pub async fn select_by_id(
    pool: &MySqlPool,
    id: i32
) -> DBResult<Option<Verification>> {
    Ok(sqlx::query_as::<_,Verification>("SELECT * FROM verifications WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

#[instrument(err,skip_all)]
pub async fn delete_by_id(
    pool: &MySqlPool,
    id: i32
) -> DBResult<()>{
    sqlx::query("DELETE FROM verifications WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}