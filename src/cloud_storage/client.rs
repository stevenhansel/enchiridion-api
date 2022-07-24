use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
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
    pub filetype: String,
    pub key: String,
    pub path: String,
}

impl TmpFile {
    pub fn new(filename: String, filetype: String, key: String) -> Self {
        TmpFile {
            path: format!("./tmp/{}.{}", filename.clone(), filetype.clone()),
            filename,
            filetype,
            key,
        }
    }

    pub fn write(path: String, data: Vec<Bytes>) -> Result<(), TmpFileError> {
        if let Err(e) = create_dir_all("./tmp") {
            return Err(TmpFileError::WriteError(e.to_string()));
        }
        let mut file = match File::create(path.clone()) {
            Ok(f) => f,
            Err(e) => return Err(TmpFileError::WriteError(e.to_string())),
        };

        for bytes in data {
            if let Err(e) = file.write(&bytes) {
                return Err(TmpFileError::WriteError(e.to_string()));
            };
        }

        Ok(())
    }
    pub fn remove(&self) -> Result<(), TmpFileError> {
        if let Err(e) = remove_file(self.path.clone()) {
            return Err(TmpFileError::RemoveError(e.to_string()));
        }

        Ok(())
    }

    pub fn name(&self) -> String {
        format!("{}.{}", self.filename, self.filetype)
    }

    pub fn key(&self) -> String {
        format!("{}/{}.{}", self.key, self.filename, self.filetype)
    }
}

pub enum CloudStorageError {
    PresignedRequestError(String),
    ReadFileError(String),
    UploadError(String),
    DeleteFileError(String),
}

impl std::fmt::Display for CloudStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudStorageError::PresignedRequestError(message) => write!(f, "{}", message),
            CloudStorageError::ReadFileError(message) => write!(f, "{}", message),
            CloudStorageError::UploadError(message) => write!(f, "{}", message),
            CloudStorageError::DeleteFileError(message) => write!(f, "{}", message),
        }
    }
}

#[async_trait]
pub trait CloudStorageClient {
    async fn get_object(&self, key: String) -> Result<String, CloudStorageError>;
    async fn upload(&self, file: TmpFile) -> Result<(), CloudStorageError>;
}

pub struct Client {
    _storage: Box<dyn CloudStorageClient + Send + Sync + 'static>,
}

impl Client {
    pub fn new(_storage: Box<dyn CloudStorageClient + Send + Sync + 'static>) -> Self {
        Client { _storage }
    }

    pub async fn get_object(&self, key: String) -> Result<String, CloudStorageError> {
        self._storage.get_object(key).await
    }

    pub async fn upload(&self, file: TmpFile) -> Result<(), CloudStorageError> {
        self._storage.upload(file).await
    }
}
