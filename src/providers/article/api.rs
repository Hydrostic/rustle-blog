use rustle_derive::handler_with_instrument;
use salvo::prelude::*;

use crate::{utils::{error_handling::AppResult, response::ResponseUtil}, middlewares::auth::auth_middleware, providers::auth::service::check_permission_api};
// use crate::permission_check;
pub fn init() -> Router {

        Router::with_path("/v1/article")
            // .push(Router::with_path("/get").post(sign_in))
            .push(
                Router::with_path("/create")
                    .hoop(auth_middleware)
                    .post(create),
            )
}
// #[derive(Debug, Validate, Deserialize, Extractible)]
// #[salvo(extract(default_source(from = "body", parse = "json")))]
// struct SignInReq<'a> {
//     #[validate(length(min = 1, max = 50))]
//     pub name: &'a str,
//     #[validate(length(min = 1, max = 50))]
//     pub password: &'a str,
// }

#[handler_with_instrument]
async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    check_permission_api(depot.session().unwrap().get::<i32>("user_id"), "CREATE_ARTICLE").await?;

    res.ok()
}