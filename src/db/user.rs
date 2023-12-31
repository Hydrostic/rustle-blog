use rbatis::executor::Executor;
use serde::{Serialize,Deserialize};
use rbatis::rbdc::db::ExecResult;
#[derive(Deserialize,Serialize,Debug)]
pub struct User{
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: Option<i16>
}
#[derive(Deserialize,Serialize,Debug)]
pub struct UserIdName{
    pub id: i32,
    pub name: String,
}

#[html_sql("src/db/user.html")]
pub async fn select_by_identity_with_password(
    rb: &dyn Executor,
    identity: &str
) -> rbatis::Result<Option<User>> {
    impled!()
}

#[html_sql("src/db/user.html")]
pub async fn select_by_id(
    rb: &dyn Executor,
    id: i32
) -> rbatis::Result<Option<User>> {
    impled!()
}
#[html_sql("src/db/user.html")]
pub async fn select_by_id_with_password(
    rb: &dyn Executor,
    id: i32
) -> rbatis::Result<Option<User>> {
    impled!()
}
#[html_sql("src/db/user.html")]
pub async fn select_by_email(
    rb: &dyn Executor,
    email: &str
) -> rbatis::Result<Option<User>> {
    impled!()
}
#[html_sql("src/db/user.html")]
pub async fn create(
    rb: &dyn Executor,
    name: &str,
    email: &str,
    password: &str,
    role: i16
) -> rbatis::Result<ExecResult>{
    impled!()
}