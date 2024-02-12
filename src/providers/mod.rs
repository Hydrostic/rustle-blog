use ntex::web;

use crate::{get_config, middlewares};

pub mod auth;
pub mod user;
pub mod article;
pub mod renderer;


pub async fn run() -> std::io::Result<()>{
    let http_config = get_config!(http);
        web::HttpServer::new(|| {
            web::App::new().wrap(middlewares::Log)
            .configure(auth::api::init)
            .configure(user::api::init)
            .configure(article::api::init)
        })
        .bind((http_config.host.as_str(), http_config.port))?
        .run()
        .await?;

        Ok(())
}
