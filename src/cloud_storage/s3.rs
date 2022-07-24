use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_s3::{presigning::config::PresigningConfig, types::ByteStream, Client};

use super::{CloudStorageClient, CloudStorageError, TmpFile};

pub struct S3Adapter {
    client: Client,
    bucket: String,
}

impl S3Adapter {
    pub fn new(client: Client, bucket: String) -> Self {
        S3Adapter { client, bucket }
    }
}

#[async_trait]
impl CloudStorageClient for S3Adapter {
    async fn get_object(&self, key: String) -> Result<String, CloudStorageError> {
        let config = match PresigningConfig::expires_in(Duration::from_secs(900)) {
            Ok(conf) => conf,
            Err(e) => return Err(CloudStorageError::PresignedRequestError(e.to_string())),
        };
        let presigned_request = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(key)
            .presigned(config)
            .await;
        let response = match presigned_request {
            Ok(res) => res,
            Err(e) => return Err(CloudStorageError::PresignedRequestError(e.to_string())),
        };

        Ok(response.uri().to_string())
    }

    async fn upload(&self, file: TmpFile) -> Result<(), CloudStorageError> {
        let body = match ByteStream::from_path(Path::new(&file.path)).await {
            Ok(body) => body,
            Err(e) => return Err(CloudStorageError::ReadFileError(e.to_string())),
        };

        let upload = self
            .client
            .put_object()
            .bucket(self.bucket.clone())
            .key(format!("{}/{}", file.key, file.name()))
            .body(body)
            .send()
            .await;
        if let Err(e) = upload {
            return Err(CloudStorageError::UploadError(e.to_string()));
        }

        if let Err(e) = file.remove() {
            return Err(CloudStorageError::DeleteFileError(e.to_string()));
        };

        Ok(())
    }
}
