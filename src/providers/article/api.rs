use chrono::Utc;
use rustle_derive::handler_with_instrument;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use crate::utils::error_handling::AppError;
use crate::utils::{error_handling::AppResult, response::ResponseUtil};
use crate::middlewares::auth::auth_middleware;
use crate::providers::auth::service::check_permission_api;
use crate::db::{get_db_pool, article::{ArticleFilterable, ArticleSortable, ArticlePublicBrief}};
use crate::db::{article as articleDao, article::Article};

pub fn init() -> Router {

        Router::with_path("/v1/article")
            .push(
                Router::with_path("/create")
                    .hoop(auth_middleware)
                    .post(create),
            )
            .push(
                Router::with_path("/list")
                    .post(list),
            )
}

#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct CreateReq<'a> {
    #[validate(length(min = 1, max = 255))]
    pub title: &'a str,
    #[validate(length(min = 1, max = 255))]
    pub alias: &'a str,
    #[validate(range(min = 1, max = 3))]
    pub public_state: i16,
    pub is_pinned: bool,
    pub is_commentable: bool,
    #[validate(length(min = 0, max = 1073741823))]
    // the max size of mysql longtext is 4,294,967,295 bytes, for the worst case, every character takes 4 bytes
    // that would be 1,073,741,823 characters
    pub draft: &'a str,
    #[validate(length(min = 0, max = 1073741823))]
    pub generated: String,
}
#[handler_with_instrument]
async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let user_id = depot.session().unwrap().get::<i32>("user_id").unwrap();
    check_permission_api(Some(user_id), "CREATE_ARTICLE").await?;
    let req_data = req.parse_json::<CreateReq>().await?;
    req_data.validate()?;
    let draft_content_id = articleDao::save_content(get_db_pool(), req_data.draft).await?;
    let content_id = articleDao::save_content(get_db_pool(), &ammonia::clean(&req_data.generated)).await?;
    let article_object = Article{
        author: user_id,
        title: req_data.title.to_string(),
        alias: req_data.alias.to_string(),
        is_pinned: req_data.is_pinned,
        is_commentable: req_data.is_commentable,
        draft_content_id,
        content_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        ..Default::default()
    };
    let id = articleDao::create(get_db_pool(), &article_object).await?;
    res.data(Json(
        json!({
            "id": id
        })
    ))
}

#[derive(Debug, Validate, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
struct ListReq {
    filter: Vec<ArticleFilterable>,
    sort: Vec<ArticleSortable>,
    #[validate(range(min = 0,max = 100))]
    limit: i32,
    #[validate(range(min = 1,))]
    page: i32
}
#[derive(Debug, Serialize)]
struct ListRes {
    total: i32,
    articles: Vec<ArticlePublicBrief>
}
#[handler_with_instrument]
async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let req_data = req.parse_json::<ListReq>().await?;
    req_data.validate()?;
    let db_res = articleDao::list::<ArticlePublicBrief>(get_db_pool(), 
        req_data.limit, 
        (req_data.page-1).checked_mul(req_data.limit).ok_or(AppError::UnexpectedError(StatusCode::BAD_REQUEST, "math.overflow".to_string()))?, 
        req_data.filter, 
        req_data.sort).await?;
    res.data(
        Json(ListRes{
            total: db_res.0,
            articles: db_res.1
        })
    )
}
