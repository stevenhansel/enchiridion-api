pub enum RedisErrorCode {
    StreamGroupAlreadyExists,
}

impl std::fmt::Display for RedisErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RedisErrorCode::StreamGroupAlreadyExists => write!(f, "BUSYGROUP"),
        }
    }
}
