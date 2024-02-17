use ntex::web;
use ntex::web::types::Payload;
use ntex::util::BytesMut;
use once_cell::sync::Lazy;
use tracing::error;
use crate::{get_args, middlewares};
use crate::types::err::{AppError, AppResult, GlobalUserError};


const UNKNOWN_USER_ID: i32 = 0;
pub fn get_user_id(req: &web::HttpRequest) -> i32{
    req.extensions().get::<middlewares::auth::UserIdentity>()
        .map(|t| t.id).unwrap_or(UNKNOWN_USER_ID)
}
pub fn check_content_length(req: &web::HttpRequest, required_size: usize) -> AppResult<()>{
    let length_header = req.headers().get("content-length").ok_or(GlobalUserError::PayloadLengthRequired)?
        .to_str().map_err(|_| GlobalUserError::PayloadLengthRequired)?;
    let size: usize = length_header.parse().map_err(|_| GlobalUserError::PayloadLengthRequired)?;
    if size > required_size{
        Err(GlobalUserError::PayloadTooLarge.into())
    } else {
        Ok(())
    }
}
use trie_rs::{Trie, TrieBuilder};

pub static ALLOWED_IMAGE_MIME: Lazy<Trie<u8>> = Lazy::new(|| {
    let mut builder = TrieBuilder::new();
    builder.push("image/jpeg");
    builder.push("image/png");
    builder.push("image/gif");
    builder.push("image/bmp");
    builder.push("image/webp");
    builder.push("image/tiff");
    builder.build()
});
pub fn check_mime<'a>(req: &'a web::HttpRequest, mimes: &Trie<u8>) -> AppResult<&'a str>{
    let mime_str = req.headers().get("content-type").ok_or(GlobalUserError::InvalidMime)?
        .to_str().map_err(|_| GlobalUserError::InvalidMime)?;
    if mimes.exact_match(mime_str){
        Ok(mime_str)
    } else {
        Err(GlobalUserError::InvalidMime.into())
    }
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