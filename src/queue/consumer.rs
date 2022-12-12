use redis::{
    streams::{StreamKey, StreamPendingReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, RedisResult, Value,
};
use serde::de::DeserializeOwned;

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

#[derive(Debug)]
pub enum ConsumerError {
    GroupAlreadyExists(&'static str),
    EmptyStream(&'static str),
    ApplicationError(String),
    RedisError(redis::RedisError),
}

impl std::fmt::Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConsumerError::GroupAlreadyExists(message) => write!(f, "{}", message),
            ConsumerError::EmptyStream(message) => write!(f, "{}", message),
            ConsumerError::ApplicationError(message) => write!(f, "{}", message),
            ConsumerError::RedisError(e) => write!(f, "{}", e.to_string()),
        }
    }
}

pub struct Consumer {
    client: deadpool_redis::Pool,
    queue_name: String,
    group_name: &'static str,
    consumer_name: &'static str,

    is_group_exist: bool,
}

impl Consumer {
    pub fn new(client: deadpool_redis::Pool, queue_name: String) -> Consumer {
        Consumer {
            client,
            queue_name,
            group_name: "main-group",
            consumer_name: "main-consumer",

            is_group_exist: false,
        }
    }

    pub async fn redis(&mut self) -> Result<deadpool_redis::Connection, ConsumerError> {
        let mut redis = self
            .client
            .get()
            .await
            .expect("Cannot get redis connection");

        if !self.is_group_exist {
            if let Err(e) = redis
                .xgroup_create_mkstream::<String, String, String, ()>(
                    self.queue_name.clone(),
                    self.group_name.to_string(),
                    "$".into(),
                )
                .await
            {
                let code = match e.code() {
                    Some(code) => code,
                    None => return Err(ConsumerError::RedisError(e)),
                };

                if code != RedisErrorCode::StreamGroupAlreadyExists.to_string() {
                    return Err(ConsumerError::GroupAlreadyExists("Group already exists"));
                }

                self.is_group_exist = true;
            };
        }

        Ok(redis)
    }

    pub fn parse<T: DeserializeOwned>(
        &self,
        data: Vec<StreamKey>,
    ) -> Result<Vec<(String, T)>, ConsumerError> {
        let mut raw: Vec<(String, String)> = vec![];

        for res in data {
            for stream_id in res.ids {
                if let Some(data) = stream_id.map.get("data") {
                    if let Value::Data(buffer) = data {
                        if let Ok(value) = std::str::from_utf8(buffer) {
                            raw.push((stream_id.id, value.to_string()));
                        }
                    }
                }
            }
        }

        let mut result: Vec<(String, T)> = vec![];

        for (message_id, data) in raw {
            match serde_json::from_str::<T>(data.as_str()) {
                Ok(payload) => result.push((message_id, payload)),
                Err(e) => return Err(ConsumerError::ApplicationError(e.to_string())),
            };
        }

        Ok(result)
    }

    pub fn parse_raw(&self, data: Vec<StreamKey>) -> Result<Vec<(String, String)>, ConsumerError> {
        let mut raw: Vec<(String, String)> = vec![];

        for res in data {
            for stream_id in res.ids {
                if let Some(data) = stream_id.map.get("data") {
                    if let Value::Data(buffer) = data {
                        if let Ok(value) = std::str::from_utf8(buffer) {
                            raw.push((stream_id.id, value.to_string()));
                        }
                    }
                }
            }
        }

        let mut result: Vec<(String, String)> = vec![];

        for (message_id, data) in raw {
            result.push((message_id, data))
        }

        Ok(result)
    }

    pub async fn consume<T: DeserializeOwned>(
        &mut self,
    ) -> Result<Vec<(String, T)>, ConsumerError> {
        let mut redis = self.redis().await?;

        let opts = StreamReadOptions::default()
            .group(self.group_name.to_string(), self.consumer_name.to_string())
            .block(0)
            .count(1);
        let result: RedisResult<StreamReadReply> = redis
            .xread_options(&[self.queue_name.clone()], &[">"], &opts)
            .await;

        let keys = match result {
            Ok(r) => r.keys,
            Err(e) => return Err(ConsumerError::RedisError(e)),
        };

        Ok(self.parse::<T>(keys)?)
    }

    pub async fn read_by_message_id<T: DeserializeOwned>(
        &mut self,
        message_id: String,
    ) -> Result<Vec<(String, T)>, ConsumerError> {
        let mut redis = self.redis().await?;

        let opts = StreamReadOptions::default()
            .group(self.group_name.to_string(), self.consumer_name.to_string())
            .count(1);
        let result: RedisResult<StreamReadReply> = redis
            .xread_options(&[self.queue_name.clone()], &[message_id], &opts)
            .await;

        let keys = match result {
            Ok(r) => r.keys,
            Err(e) => return Err(ConsumerError::RedisError(e)),
        };

        Ok(self.parse::<T>(keys)?)
    }

    pub async fn consume_raw(&mut self) -> Result<Vec<(String, String)>, ConsumerError> {
        let mut redis = self.redis().await?;

        let opts = StreamReadOptions::default()
            .group(self.group_name.to_string(), self.consumer_name.to_string())
            .block(0)
            .count(1);
        let result: RedisResult<StreamReadReply> = redis
            .xread_options(&[self.queue_name.clone()], &[">"], &opts)
            .await;

        let keys = match result {
            Ok(r) => r.keys,
            Err(e) => return Err(ConsumerError::RedisError(e)),
        };

        Ok(self.parse_raw(keys)?)
    }

    pub async fn read_raw_by_message_id(
        &mut self,
        message_id: String,
    ) -> Result<Vec<(String, String)>, ConsumerError> {
        let mut redis = self.redis().await?;

        let opts = StreamReadOptions::default()
            .group(self.group_name.to_string(), self.consumer_name.to_string())
            .count(1);
        let result: RedisResult<StreamReadReply> = redis
            .xread_options(&[self.queue_name.clone()], &[message_id], &opts)
            .await;

        let keys = match result {
            Ok(r) => r.keys,
            Err(e) => return Err(ConsumerError::RedisError(e)),
        };

        Ok(self.parse_raw(keys)?)
    }



    pub async fn get_pending_message_id(&mut self) -> Result<Option<String>, ConsumerError> {
        let mut redis = self.redis().await?;

        let result: RedisResult<StreamPendingReply> = redis
            .xpending(self.queue_name.clone(), self.group_name.clone())
            .await;

        let reply = match result {
            Ok(r) => match r {
                StreamPendingReply::Data(reply) => reply,
                StreamPendingReply::Empty => return Ok(None),
            },
            Err(e) => return Err(ConsumerError::RedisError(e)),
        };

        Ok(Some(reply.start_id))
    }

    pub async fn ack(&mut self, message_id: String) -> Result<(), ConsumerError> {
        let mut redis = self.redis().await?;

        if let Err(e) = redis
            .xack::<String, String, String, ()>(
                self.queue_name.clone(),
                self.group_name.to_string(),
                &[message_id],
            )
            .await
        {
            return Err(ConsumerError::RedisError(e));
        }

        Ok(())
    }
}
