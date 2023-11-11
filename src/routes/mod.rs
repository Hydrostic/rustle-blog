use salvo::Router;

pub mod auth;


pub fn init(router: Router) -> Router {
    auth::init(router)
}