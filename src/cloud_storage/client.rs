use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
    sync::Arc,
};

use actix_web::web::Bytes;
use async_trait::async_trait;

pub enum TmpFileError {
    WriteError(String),
    RemoveError(String),
}

impl std::fmt::Display for TmpFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TmpFileError::WriteError(message) => write!(f, "{}", message),
            TmpFileError::RemoveError(message) => write!(f, "{}", message),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TmpFile {
    pub filename: String,
    pub path: String,
}

impl TmpFile {
    pub fn new(filename: String) -> Self {
        TmpFile {
            filename: filename.clone(),
            path: format!("./tmp/{}", filename.clone()),
        }
    }

    pub fn write(path: String, data: Bytes) -> Result<(), TmpFileError> {
        if let Err(e) = create_dir_all("./tmp") {
            return Err(TmpFileError::WriteError(e.to_string()));
        }
        let mut file = match File::create(path.clone()) {
            Ok(f) => f,
            Err(e) => return Err(TmpFileError::WriteError(e.to_string())),
        };

        if let Err(e) = file.write_all(&data) {
            return Err(TmpFileError::WriteError(e.to_string()));
        };

        Ok(())
    }
    pub fn remove(&self) -> Result<(), TmpFileError> {
        if let Err(e) = remove_file(self.path.clone()) {
            return Err(TmpFileError::RemoveError(e.to_string()));
        }

        Ok(())
    }
}

pub enum CloudStorageError {}

#[async_trait]
pub trait CloudStorageClient {
    //    async fn upload(&self) -> Result<String, CloudStorageError>;
}

pub struct Client {
    _storage: Arc<dyn CloudStorageClient + Send + Sync + 'static>,
}

impl Client {
    pub fn new(_storage: Arc<dyn CloudStorageClient + Send + Sync + 'static>) -> Self {
        Client { _storage }
    }
}
