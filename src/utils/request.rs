use ntex::web;
use ntex::web::types::Payload;
use ntex::util::BytesMut;
use tracing::error;
use crate::{get_args, middlewares};
use crate::types::err::{AppError, GlobalUserError};


const UNKNOWN_USER_ID: i32 = 0;
pub fn get_user_id(req: web::HttpRequest) -> i32{
    req.extensions().get::<middlewares::auth::UserIdentity>()
        .map(|t| t.id).unwrap_or(UNKNOWN_USER_ID)
}


pub struct RequestPayload<'a>{
    payload: &'a mut Payload,
    b: BytesMut
}

impl<'a> RequestPayload<'a>{
    pub fn new(payload: &'a mut Payload) -> Self{
        Self{payload, b: BytesMut::new()}
    }   
    pub async fn parse<'de, T>(&'de mut self) -> Result<T, AppError>
    where
        T: serde::de::Deserialize<'de>,
    {
        while let Some(item) = ntex::util::stream_recv(self.payload).await {
            self.b.extend_from_slice(&item.map_err(|_| GlobalUserError::InvalidPayload)?);
        }
        serde_json::from_slice(&self.b).map_err(|e| {
            if get_args!(debug) { 
                error!("failed to deseralize payload: {:?}", e);
            }
            GlobalUserError::InvalidPayload.into()
        })

    }
}