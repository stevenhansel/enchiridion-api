use super::CloudStorageClient;

use aws_sdk_s3::types::ByteStream;

pub struct S3 {}

impl CloudStorageClient for S3 {}
