use std::borrow::Borrow;
use std::error::Error;
use std::path::Path;

use futures_util::Stream;
use tokio::io::AsyncRead;
use tokio_util::bytes::Bytes;
use crate::types::err::AppResult;
use super::local::LocalFs;
use super::{FsUserError, FS_POLICY_CACHE};

pub enum FsProvider{
    LocalProvider(LocalFs),
    ExtensionProvider{
        // js running context etc    
    }
}

fn check_path(p: &Path) -> bool{
    return !p.strip_prefix("/").map_or(true, |x| x.starts_with("__")) && p.is_absolute();
}
// Because this system designed to run under linux, it will not take rules of filename in windows into consideration
// actually in windows, the entire path need to 256 chars at maximum, and should not contains chars like <, > etc.
// Warning: the func will run a test to ensure that it does not contain relative path like .. to avoid path injection and start with /
// also, the name of root directory is not allowed to start with __ (upload_file_internal do not has constraints on this)
impl FsProvider {
    pub async fn upload_file<T: AsyncRead + Unpin>( 
        stream: &mut T, 
        policy_id: i32,
        user_id: i32,
        path: &Path) -> AppResult<u64>{
            if !check_path(path){
                return Err(FsUserError::PathNameNotValid.into());
            }
            Self::upload_file_internal(stream, policy_id, user_id, path).await
        }
        pub async fn upload_file_internal<T: AsyncRead + Unpin>( 
            stream: &mut T, 
            policy_id: i32,
            user_id: i32,
            path: &Path) -> AppResult<u64>{
                
                let policy_spec = FS_POLICY_CACHE.get(&policy_id).ok_or(FsUserError::PolicyNotFound)?;
                match policy_spec.instance.borrow(){
                    FsProvider::LocalProvider(p) => p.upload_file(stream, user_id, path).await,
                    FsProvider::ExtensionProvider {  } => todo!(),
                }
            }
            pub async fn get_file( 
                policy_id: i32,
                user_id: i32,
                path: &Path) 
                -> AppResult<impl AsyncRead + Unpin>{
                    
                    let policy_spec = FS_POLICY_CACHE.get(&policy_id).ok_or(FsUserError::PolicyNotFound)?;
                    match policy_spec.instance.borrow(){
                        FsProvider::LocalProvider(p) => p.get_file(user_id, path).await,
                        FsProvider::ExtensionProvider {  } => todo!(),
                    }
                }
}