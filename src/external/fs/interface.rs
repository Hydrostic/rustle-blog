use std::borrow::Borrow;

use tokio::io::AsyncRead;
use crate::types::err::AppResult;
use super::local::LocalFs;
use super::{FsUserError, FS_POLICY_CACHE};


pub enum FsProvider{
    LocalProvider(LocalFs),
    ExtensionProvider{
        // js running context etc    
    }
}

impl FsProvider {
    pub async fn upload_file<T: AsyncRead + Unpin>( 
        stream: &mut T, 
        policy_id: i32,
        name: &str, 
        alias: &str) -> AppResult<u64>{
            let policy_spec = FS_POLICY_CACHE.get(&policy_id).ok_or(FsUserError::PolicyNotFound)?;
            match policy_spec.instance.borrow(){
                FsProvider::LocalProvider(p) => p.upload_file(stream, name, alias).await,
                FsProvider::ExtensionProvider {  } => todo!(),
            }
        }
}