use std::borrow::Cow;
use chrono::Utc;
use ntex::web::{self, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::providers::auth::service::check_permission_api;
use crate::db::{get_db_pool, article::{ArticleFilterable, ArticleSortable, ArticlePublicBrief}};
use crate::db::{article as articleDao, article::Article};
use crate::utils::request::{get_user_id, RequestPayload};
use validator::Validate;
use crate::types::err::AppResult;
use crate::types::err::GlobalUserError::TooMaxParameter;
use crate::middlewares::Auth;

pub fn init(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/v1/article")
            .service(list)
    );
    cfg.service(
        web::scope("/v1/article")
            .wrap(Auth)
            .service(create)
    );
}

#[derive(Debug, Validate, Deserialize)]
struct CreateReq<'a> {
    #[validate(length(min = 1, max = 255))]
    #[serde(borrow)]
    pub title: Cow<'a,str>,
    #[validate(length(min = 1, max = 255))]
    #[serde(borrow)]
    pub alias: Cow<'a,str>,
    #[validate(range(min = 1, max = 3))]
    pub public_state: i16,
    pub is_pinned: bool,
    pub is_commentable: bool,
    #[validate(length(min = 0, max = 1073741823))]
    // the max size of mysql longtext is 4,294,967,295 bytes, for the worst case, every character takes 4 bytes
    // that would be 1,073,741,823 characters
    #[serde(borrow)]
    pub draft: Cow<'a,str>,
    #[validate(length(min = 0, max = 1073741823))]
    #[serde(borrow)]
    pub generated: Cow<'a,str>,
}
#[web::post("/create")]
async fn create(req: web::HttpRequest, mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: CreateReq<'_> = payload.parse().await?;
    req_data.validate()?;
    
    let user_id = get_user_id(&req);
    check_permission_api(Some(user_id), "CREATE_ARTICLE").await?;
    let draft_content_id = articleDao::save_content(get_db_pool(), &req_data.draft).await?;
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
    Ok(web::HttpResponse::Ok().body(
        json!({
            "id": id
        })
    ))
}

#[derive(Debug, Validate, Deserialize)]
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
#[web::post("/list")]
async fn list(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: ListReq = payload.parse().await?;
    req_data.validate()?;
    let db_res = articleDao::list::<ArticlePublicBrief>(get_db_pool(),
        req_data.limit,
        (req_data.page-1).checked_mul(req_data.limit).ok_or(TooMaxParameter)?,
        req_data.filter,
        req_data.sort).await?;
    Ok(web::HttpResponse::Ok().json(
        &ListRes{
            total: db_res.0,
            articles: db_res.1
        }
    ))
}

