use std::sync;
use ntex::web::{self, Responder};
use crate::db::rbac::{self as rbacDao, role_vec_to_str, Role, DEFAULT_ROLE_STR};
use crate::middlewares::Auth;
use crate::providers::auth::service::check_permission_api;
use crate::types::err::GlobalUserError::{CredentialUnauthorized, TooMaxParameter};
use crate::db::{user as userDao, get_db_pool};
use crate::utils::request::{get_user_id, RequestPayload};
use crate::utils::{paseto, password_salt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use serde_json::json;
use validator::Validate;
use crate::{get_args, get_config};
use crate::types::err::AppResult;
use crate::utils::paseto::generate_access_token;

pub static ONCE_INIT: sync::Once = sync::Once::new();
pub fn init(cfg: &mut web::ServiceConfig){
        cfg.service(
            web::scope("/v1/auth")
                .service(sign_in)
                .configure(|r| {
                    if get_args!(debug){
                        r.service(__test_add_user__)
                        .service(__test_get_token__);
                    }
                }).service(
                    web::scope("/").wrap(Auth)
                            .service(modify_user_roles)
                            .service(remove_role)
                            .service(list_roles)
                            .service(add_role)
                )
        );
        if get_args!(debug) {
            ONCE_INIT.call_once(|| {
                println!("\x1b[31m!!!@@@ warning @@@!!!\n\
                        debug mode api(__test_add_user__, __test_get_token__) added\n\
                        in no case should you enable it in the production\n\
                        !!!!!!!!!!!!!!!!!!!!!!\x1b[0m");
            });
        }
}
#[derive(Debug, Validate, Deserialize)]
struct SignInReq<'a> {
    #[validate(length(min = 1, max = 50))]
    pub name: Cow<'a, str>,
    #[validate(length(min = 1, max = 100))]
    pub password: Cow<'a, str>,
}
#[web::post("/sign_in")]
async fn sign_in(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: SignInReq<'_> = payload.parse().await?;
    req_data.validate()?;
    let user_data = userDao::select_by_identity_with_password(get_db_pool(), &req_data.name).await?;
    if user_data.is_none() {
        return Err(CredentialUnauthorized.into());
    }

    let user_data = user_data.unwrap();
    if !password_salt::compare_password(&user_data.password.unwrap(), &req_data.password) {
        return Err(CredentialUnauthorized.into());
    }
    let token = paseto::generate_access_token(&get_config!(security).auth_token_secret, user_data.id)?;
    Ok(web::HttpResponse::Ok().body(
        json!({
            "id": user_data.id,
            "name": user_data.name,
            "email": user_data.email,
            "access_token": token
        })
    ))
}
#[derive(Deserialize, Debug)]
struct ModifyUserRolesReq{
    pub roles: Vec<i32>
}
#[web::post("/modify_user_roles")]
async fn modify_user_roles(req_data: web::types::Json<ModifyUserRolesReq>, req: web::HttpRequest) -> AppResult<impl Responder> {
    let user_id = get_user_id(&req);
    check_permission_api(Some(user_id), "MANAGE_ROLE").await?;
    let req_data = req_data.into_inner();
    
    rbacDao::update_user_roles(get_db_pool(), user_id, &role_vec_to_str(req_data.roles)?).await?;
    Ok(web::HttpResponse::Ok().finish())
}
#[web::post("/remove_role/{role_id}")]
async fn remove_role(path: web::types::Path<i32>, req: web::HttpRequest) -> AppResult<impl Responder> {
    check_permission_api(Some(get_user_id(&req)), "MANAGE_ROLE").await?;
    let role_id = path.into_inner();
    rbacDao::delete_role(get_db_pool(), role_id).await??;
    super::service::delete_role_cache(role_id);
    Ok(web::HttpResponse::Ok().finish())
}
#[derive(Debug, Validate, Deserialize)]
struct RoleListReq {
    #[validate(range(min = 0, max = 100))]
    limit: i32,
    #[validate(range(min = 1,))]
    page: i32,
}
#[derive(Serialize)]
struct RoleListRes {
    total: i32,
    roles: Vec<Role>,
}
#[web::get("/list_roles")]
async fn list_roles(req: web::HttpRequest, req_data: web::types::Json<RoleListReq>) -> AppResult<impl Responder> {
    check_permission_api(Some(get_user_id(&req)), "MANAGE_ROLE").await?;
    let db_data = rbacDao::list_roles(get_db_pool(), 
    req_data.limit, 
    (req_data.page - 1).checked_mul(req_data.limit).ok_or(TooMaxParameter)?).await?;
    req_data.validate()?;
    Ok(web::HttpResponse::Ok().json(
        &RoleListRes{ 
            total: db_data.0,
            roles: db_data.1,
        }
    ))
}
#[derive(Debug, Validate, Deserialize)]
struct AddRoleReq<'a> {
    #[validate(length(min = 0, max = 50))]
    name: Cow<'a, str>,
    #[validate(length(min = 0, max = 50))]
    alias: &'a str,
    permissions: Vec<&'a str>
    // todo: further check
}
#[web::post("/add_role")]
async fn add_role(req: web::HttpRequest, mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: AddRoleReq<'_> = payload.parse().await?;

    req_data.validate()?;
    check_permission_api(Some(get_user_id(&req)), "MANAGE_ROLE").await?;
    let _ = rbacDao::add_role(get_db_pool(), &req_data.name, req_data.alias, req_data.permissions).await?;
    Ok(web::HttpResponse::Ok().finish())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModifyRolePermissionAction{
    Remove,
    Add
}
#[derive(Debug, Validate, Deserialize)]
struct ModifyRolePermissionReq<'a> {
    role: i32,
    permission: &'a str,
    action: ModifyRolePermissionAction
}
#[web::post("/modify_role_permission")]
async fn modify_role_permission(req: web::HttpRequest, mut payload: web::types::Payload) -> AppResult<impl Responder>{
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: ModifyRolePermissionReq<'_> = payload.parse().await?;

    req_data.validate()?;
    check_permission_api(Some(get_user_id(&req)), "MANAGE_ROLE").await?;
    match req_data.action{
        ModifyRolePermissionAction::Remove => rbacDao::delete_role_permission(get_db_pool(), req_data.role, req_data.permission).await?,
        ModifyRolePermissionAction::Add => rbacDao::add_role_permission(get_db_pool(), req_data.role, req_data.permission).await?
    };
    
    Ok(web::HttpResponse::Ok().finish())
}

#[derive(Debug, Validate, Deserialize)]
struct TestAddUser<'a> {
    #[validate(length(min = 1, max = 50))]
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[validate(length(min = 1, max = 100))]
    pub email: &'a str, // as email should not contain any special characters, it's ok to use raw str
    #[validate(length(min = 1, max = 100))]
    #[serde(borrow)]
    pub password: Cow<'a, str>,
}
// debug mode only
#[web::post("/__test_add_user__")]
async fn __test_add_user__(mut payload: web::types::Payload) -> AppResult<impl Responder> {
    let mut payload = RequestPayload::new(&mut payload);
    let req_data: TestAddUser<'_> = payload.parse().await?;
    req_data.validate()?;
    let hashed_password = password_salt::generate_password(&req_data.password, &get_config!(security).password_salt)?;
    let user_insert_res = userDao::create(
        get_db_pool(),
        &req_data.name,
        &req_data.email,
        &hashed_password,
        DEFAULT_ROLE_STR
    )
    .await?;
    Ok(web::HttpResponse::Ok().body(json!({
        "id": user_insert_res
    })))
}
#[derive(Deserialize)]
struct TestGetTokenReq {
    pub id: i32
}
// debug mode only
#[web::get("/__test_get_token__")]
async fn __test_get_token__(user: web::types::Query<TestGetTokenReq>) -> AppResult<impl Responder> {
    let token = generate_access_token(&get_config!(security).auth_token_secret, user.id)?;
    Ok(web::HttpResponse::Ok().body(
        json!({
            "access_token": token
        })
    ))
}

