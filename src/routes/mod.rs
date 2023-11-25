use salvo::Router;

pub mod auth;
pub mod user;

pub fn init() -> Router{
    Router::new()
    .push(auth::init())
    .push(user::init())
}