use async_process::{Command, Stdio};
use std::{
    fs::{create_dir_all, remove_file, File},
    io::Write,
};

use actix_web::web::Bytes;
use async_trait::async_trait;

use crate::features::media::domain::CropArgs;

#[derive(Debug)]
pub enum TmpFileError {
    WriteError(String),
    RemoveError(String),
    CropError(String),
}

impl std::fmt::Display for TmpFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TmpFileError::WriteError(message) => write!(f, "{}", message),
            TmpFileError::RemoveError(message) => write!(f, "{}", message),
            TmpFileError::CropError(message) => write!(f, "{}", message),
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
            path: format!("./tmp/{}.{}", filename, filetype),
            filename,
            filetype,
            key,
        }
    }

    pub fn write(path: String, data: Vec<Bytes>) -> Result<(), TmpFileError> {
        if let Err(e) = create_dir_all("./tmp") {
            return Err(TmpFileError::WriteError(e.to_string()));
        }
        let mut file = match File::create(path) {
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

    pub async fn crop(&self, crop_args: CropArgs) -> Result<Self, TmpFileError> {
        let crop_filter_str = format!(
            r#"crop={}:{}:{}:{}"#,
            crop_args.width, crop_args.height, crop_args.x, crop_args.y
        );

        let new_filename = format!("{}_cropped", self.filename);
        let new_path = format!("./tmp/{}.{}", new_filename, self.filetype);

        if let Err(e) = Command::new("ffmpeg")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-y")
            .arg("-i")
            .arg(self.path.clone())
            .arg("-filter:v")
            .arg(crop_filter_str)
            .arg("-c:a")
            .arg("copy")
            .arg(new_path.clone())
            .output()
            .await
        {
            return Err(TmpFileError::CropError(e.to_string()));
        };

        Ok(TmpFile {
            path: new_path,
            filename: new_filename,
            filetype: self.filetype.clone(),
            key: self.key.clone(),
        })
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
