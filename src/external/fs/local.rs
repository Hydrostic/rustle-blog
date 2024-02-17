use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;
use tracing::error;

use crate::types::err::GlobalInternalError;
use crate::types::err::AppResult;
use crate::types::err::GlobalUserError;
use std::fs::File;
use std::path::Path;
pub struct LocalFs{
    pub path: String
}


use futures_util::AsyncReadExt;
impl LocalFs{
    pub const EXTENSION_NAME: &'static str = "rustle.builtin.fs.local";
    pub async fn initialize(path: String) -> Result<Self, ()>{
        let f = File::open(&path).map_err(|e| {
            error!("cannot open {path}, local file provider will not be loaded: {:?}", e);
        })?;
        let meta = File::metadata(&f).map_err(|e| {
            error!("cannot open {path}, local file provider will not be loaded: {:?}", e);
        })?;
        if !meta.is_dir() {
            error!("path is not a directory, local file provider will not be loaded");
            return Err(());
        } else if meta.permissions().readonly() {
            error!("path is readonly, local file provider will not be loaded");
            return Err(());
        }
        if path.ends_with("/") {
            Ok(LocalFs{
                path
            })
        } else {
            Ok(LocalFs{
                path: format!("{}/", path)
            })
        }
    }
    pub async fn upload_file<T: AsyncRead + Unpin>(
        &self,
        stream: &mut T,
        user_id: i32,
        path: &Path,
    ) -> AppResult<u64> {
        let file_path = Path::new(&self.path).join(Path::new(&format!("{user_id}"))).join(path);
        if let Some(parent_path) = file_path.parent(){
            tokio::fs::create_dir_all(parent_path).await.map_err(|e| {
                error!("cannot create parent dir {}: {:?}", parent_path.to_string_lossy(), e);
                GlobalInternalError::IO
            })?;
        }
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|e| {
                error!("cannot open {} for save: {:?}", file_path.to_string_lossy(), e);
                GlobalInternalError::IO
            })?;
        tokio::io::copy(stream, &mut file)
            .await
            .map_err(|e| {
                error!("cannot save file: {:?}", e);
                GlobalInternalError::IO.into()
            })
    }

    pub async fn get_file(
        &self,
        user_id: i32,
        path: &Path,
    ) -> AppResult<impl AsyncRead + Unpin> {
        let file_path = Path::new(&self.path).join(Path::new(&format!("{user_id}"))).join(path);
        let file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound{
                    GlobalUserError::NotFound.to_error()
                } else {
                    error!("cannot open {} for save: {:?}", file_path.to_string_lossy(), e);
                    GlobalInternalError::IO.to_error()
                }
            })?;
        Ok(file)
    }
}
