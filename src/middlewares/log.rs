use ntex::{web, Middleware, Service, ServiceCtx};
use tracing::{Instrument, Level, span};
use uuid::Uuid;

pub struct Log;

#[derive(Clone)]
pub struct RequestId(pub String);
impl Default for RequestId{
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl<S> Middleware<S> for Log {
    type Service = LogMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        LogMiddleware { service }
    }
}

pub struct LogMiddleware<S> {
    service: S,
}

impl<S, Err> Service<web::WebRequest<Err>> for LogMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let request_id = RequestId(Uuid::new_v4().to_string());
        req.extensions_mut().insert(request_id.clone());
        let res = ctx.call(&self.service, req).instrument(
                        span!(
                            Level::INFO,
                            "weblog",
                            request_id = request_id.0
                        )
                    ).await?;
        Ok(res)
    }
}