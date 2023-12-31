use rbatis::rbdc::datetime::DateTime;
use serde::{Serialize,Deserialize};
use rbatis::executor::Executor;
use rbatis::rbdc::db::ExecResult;

#[derive(Deserialize,Serialize,Debug)]
pub struct Verification{
    pub id: i32,
    pub user: i32,
    pub identity: String,
    pub random_code: String,
    pub action: i16,
    pub created_at: DateTime
}
#[html_sql("src/db/verification.html")]
pub async fn create(
    rb: &dyn Executor,
    user: i32,
    identity: &str,
    random_code: &str,
    action: i16,
) -> rbatis::Result<ExecResult>{
    impled!()
}
