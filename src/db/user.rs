use rbatis::executor::Executor;
use serde::{Serialize,Deserialize};
use rbatis::rbdc::db::ExecResult;
#[derive(Deserialize,Serialize,Debug)]
pub struct User{
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String
}
#[derive(Deserialize,Serialize,Debug)]
pub struct UserIdName{
    pub id: i32,
    pub name: String,
}

#[html_sql("src/db/user.html")]
pub async fn select_by_identity(
    rb: &dyn Executor,
    identity: &str
) -> rbatis::Result<Option<User>> {
    impled!()
}
#[html_sql("src/db/user.html")]
pub async fn select_id_name_by_email(
    rb: &dyn Executor,
    email: &str
) -> rbatis::Result<Option<UserIdName>> {
    impled!()
}
#[html_sql("src/db/user.html")]
pub async fn create(
    rb: &dyn Executor,
    name: &str,
    email: &str,
    password: &str,
    role: u16
) -> rbatis::Result<ExecResult>{
    impled!()
}
