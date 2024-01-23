
use serde::Serialize;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlPool};
use tracing::instrument;
use rustle_derive::{FilterParams, SortParams};
use super::DBResult;

#[derive(Serialize,Debug,FromRow,Default,FilterParams,SortParams)]
pub struct Article{
    pub id: i32,
    #[filterable]
    pub author: i32,
    pub content_id: i32,
    pub draft_content_id: i32,
    pub summary_content_id: i32,
    pub template_id: i32,
    pub cover_id: i32,
    #[sortable]
    pub visits: i32,
    #[sortable]
    pub comments: i32,
    pub public_state: i16,
    pub draft_state: i16,
    #[filterable]
    pub is_pinned: bool,
    pub is_commentable: bool,
    #[sortable]
    pub created_at: DateTime<Utc>,
    #[sortable]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password: String,
    pub title: String,
    pub alias: String,
}
#[derive(Serialize,Debug,FromRow)]
pub struct ArticlePublicBrief{
    pub id: i32,
    pub author: i32,
    pub cover_id: i32,
    pub visits: i32,
    pub comments: i32,
    pub is_pinned: bool,
    pub is_commentable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub alias: String,
}
#[derive(Serialize,Debug,FromRow,Default)]
pub struct ArticleContent{
    pub id: i32,
    pub content: String,
}
#[instrument(err,skip_all)]
pub async fn save_content(pool: &MySqlPool, content: &str) -> DBResult<i32>{
    Ok(sqlx::query("INSERT INTO contents (content) VALUES (?)")
        .bind(content)
        .execute(pool)
        .await?.last_insert_id() as i32)
}
#[instrument(err,skip_all)]
pub async fn create(pool: &MySqlPool, article: &Article) -> DBResult<i32>{
    Ok(sqlx::query("INSERT INTO articles (author,content_id,draft_content_id,summary_content_id,template_id,cover_id,visits,comments,public_state,draft_state,is_pinned,is_commentable,created_at,updated_at,password,title,alias) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(article.author)
        .bind(article.content_id)
        .bind(article.draft_content_id)
        .bind(article.summary_content_id)
        .bind(article.template_id)
        .bind(article.cover_id)
        .bind(article.visits)
        .bind(article.comments)
        .bind(article.public_state)
        .bind(article.draft_state)
        .bind(article.is_pinned)
        .bind(article.is_commentable)
        .bind(article.created_at)
        .bind(article.updated_at)
        .bind(&article.password)
        .bind(&article.title)
        .bind(&article.alias)
        .execute(pool)
        .await?.last_insert_id() as i32)
}
#[instrument(err,skip_all)]
pub async fn list<T: Send + Unpin + for<'a> sqlx::FromRow<'a, sqlx::mysql::MySqlRow>>(
    pool: &MySqlPool,
    limit: i32,
    offset: i32,
    filter: Vec<ArticleFilterable>,
    sort: Vec<ArticleSortable>,
) -> DBResult<(i32,Vec<T>)>{
    let mut basic_query = "SELECT id FROM articles".to_string();
    let where_query = filter.iter().map(|f| 
        format!("{} = ?", f.get_field_name())
    ).collect::<Vec<String>>().join(" AND ");
    if where_query.len() != 0{
        basic_query.push_str(" WHERE ");
        basic_query.push_str(&where_query);
    }
    let order_query = sort.iter().map(|f| f.to_sql()).collect::<Vec<String>>().join(",");
    if order_query.len() != 0{
        basic_query.push_str(" ORDER BY ");
        basic_query.push_str(&order_query);
    }
    let final_query = format!("SELECT * FROM articles JOIN ({} LIMIT {},{})t USING(id)", basic_query, offset, limit);
    let mut instance = sqlx::query_as::<_,T>(&final_query);
    for f in filter{
        instance = f.bind_value(instance);
    }
    Ok((
            sqlx::query_as::<_,(i64,)>("SELECT count(id) FROM articles")
                .fetch_one(pool)
                .await?
                .0 as i32,
            instance
            .fetch_all(pool)
            .await?
    ))
}