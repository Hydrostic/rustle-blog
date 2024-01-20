use salvo::Router;

pub mod auth;
pub mod user;
pub mod article;

pub fn init() -> Router{
    Router::new()
    .push(auth::api::init())
    .push(user::api::init())
    .push(article::api::init())
}