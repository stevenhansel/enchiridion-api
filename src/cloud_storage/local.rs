use std::fs::create_dir_all;
use std::{fs, path::Path};

use async_trait::async_trait;

use super::{CloudStorageClient, CloudStorageError, TmpFile};

pub struct LocalAdapter {
    base_url: String,
}

impl LocalAdapter {
    pub fn new(base_url: String) -> Self {
        LocalAdapter { base_url }
    }
}

#[async_trait]
impl CloudStorageClient for LocalAdapter {
    async fn get_object(&self, key: String) -> Result<String, CloudStorageError> {
        let url = format!("{}/{}", self.base_url, key);

        Ok(url)
    }

    async fn upload(&self, file: TmpFile) -> Result<(), CloudStorageError> {
        let bytes = match fs::read(Path::new(&file.path)) {
            Ok(bytes) => bytes,
            Err(e) => return Err(CloudStorageError::ReadFileError(e.to_string())),
        };

        let static_dir = format!("./static/{}", file.key);
        if let Err(e) = create_dir_all(&static_dir) {
            return Err(CloudStorageError::UploadError(e.to_string()));
        }

        let static_path_str = format!("./static/{}/{}", file.key, file.name());
        let static_path = Path::new(&static_path_str);

        if let Err(e) = fs::write(static_path, bytes) {
            return Err(CloudStorageError::UploadError(e.to_string()));
        }

        if let Err(e) = file.remove() {
            return Err(CloudStorageError::DeleteFileError(e.to_string()));
        };

        Ok(())
    }
}
