use std::collections::HashMap;

use serde::Serialize;
use sqlx::{mysql::MySqlRow, prelude::FromRow, MySqlPool, Row};
use tracing::instrument;

use super::DBResult;
#[derive(Debug, Serialize)]
pub struct FsPolicy{
    pub id: i32,
    pub name: String,
    pub extension: String,
    pub meta: HashMap<String, String>
}
impl<'r> FromRow<'r, MySqlRow> for FsPolicy{
    fn from_row(row: &sqlx::mysql::MySqlRow) -> DBResult<Self> {
        Ok(Self{
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            extension: row.try_get("extension")?,
            meta: serde_json::from_str(row.try_get("meta")?).map_err(|e| {
                sqlx::Error::ColumnDecode { index: "meta".to_string(), source: Box::new(e) }
            })?
        })
    }
}
#[instrument(err,skip_all)]
pub async fn get_all_policies(pool: &MySqlPool) -> DBResult<Vec<FsPolicy>>{
    sqlx::query_as("SELECT * FROM fs_policies")
        .fetch_all(pool)
        .await
}