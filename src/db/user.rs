#![allow(non_upper_case_globals)]

use argon2::{Argon2, PasswordVerifier, PasswordHash};
use lazy_static::lazy_static;
use rbatis::executor::Executor;
use serde::{Serialize,Deserialize};
#[derive(Deserialize,Serialize,Debug)]
pub struct User{
    pub name: String,
    pub email: String,
    pub password: String
}

#[html_sql("src/db/user.html")]
pub async fn select_by_identity(
    rb: &dyn Executor,
    identity: &str
) -> rbatis::Result<Option<User>> {
    impled!()
}

lazy_static!{
    pub static ref Ag: Argon2<'static> = Argon2::default();
}
pub fn compare_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash){
        Ok(parsed_hash) => 
            Ag.verify_password(password.as_bytes(), &parsed_hash).is_ok(),
        Err(_) => false
    }
    
}