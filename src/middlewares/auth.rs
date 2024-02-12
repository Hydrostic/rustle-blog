use ntex::{web, Middleware, Service, ServiceCtx};
use crate::types::err::GlobalUserError::StatusUnauthorized;
use crate::utils::paseto;
use crate::get_config;
pub struct Auth;
pub struct UserIdentity{
    pub id: i32
}

impl<S> Middleware<S> for Auth {
    type Service = AuthMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AuthMiddleware { service }
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, Err> Service<web::WebRequest<Err>> for AuthMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let auth_header = req.headers().get("authorization").ok_or(StatusUnauthorized.to_error())?
            .to_str().map_err(|_| StatusUnauthorized.to_error())?;
        if !auth_header.starts_with("Bearer "){
            return Err((StatusUnauthorized.to_error()).into());
        }
        let token = auth_header[7..].trim();
        let user_id = paseto::verify_access_token(
            &(get_config!(security).auth_token_secret),
            token
        ).map_err(|_| StatusUnauthorized.to_error())?;
        req.extensions_mut().insert(UserIdentity{id: user_id});
        let res = ctx.call(&self.service, req).await?;
        Ok(res)
    }
}