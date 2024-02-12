use tokio::io::AsyncRead;
use tracing::error;

use crate::types::err::GlobalInternalError;
use crate::types::err::AppResult;
use std::fs::File;
pub struct LocalFs{
    pub path: String
}



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
        _name: &str,
        alias: &str,
    ) -> AppResult<u64> {
        let file_path = format!("{}{}", self.path, alias);
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|e| {
                error!("cannot open {file_path} for save: {:?}", e);
                GlobalInternalError::IO
            })?;
        tokio::io::copy(stream, &mut file)
            .await
            .map_err(|e| {
                error!("cannot save file: {:?}", e);
                GlobalInternalError::IO.into()
            })
    }
}
