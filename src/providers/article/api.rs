use salvo::prelude::*;

use crate::{utils::{error_handling::AppResult, response::ResponseUtil}, middlewares::auth::auth_middleware};
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

#[handler]
async fn create(_req: &mut Request, _depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    // let a = permission_check!(res, deopt, "test1");
    // let req_data = req.parse_json::<SignInReq>().await?;
    // req_data.validate()?;
    // let user_data = userDao::select_by_identity_with_password(get_db_pool(), req_data.name).await?;
    // if user_data.is_none() {
    //     return normal_response(UnauthorizedCredential("name/password"));
    // }

    // let user_data = user_data.unwrap();
    // if !password_salt::compare_password(&user_data.password.unwrap(), req_data.password) {
    //     return normal_response(UnauthorizedCredential("name/password"));
    // }
    // let mut session = Session::new();
    // session.insert("user_id", user_data.id).unwrap();
    // depot.set_session(session);
    res.ok()
}