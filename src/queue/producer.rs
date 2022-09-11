use std::collections::BTreeMap;

use redis::AsyncCommands;

pub enum ProducerError {
    RedisError(String),
}

pub struct Producer {
    client: deadpool_redis::Pool,
    queue_name: String,
}

impl Producer {
    pub fn new(client: deadpool_redis::Pool, queue_name: String) -> Producer {
        Producer { client, queue_name }
    }

    pub async fn push(&self, payload: BTreeMap<String, String>) -> Result<(), ProducerError> {
        let mut conn = self
            .client
            .get()
            .await
            .expect("Cannot get redis connection");

        if let Err(e) = conn
            .xadd_map::<String, String, BTreeMap<String, String>, ()>(
                self.queue_name.clone(),
                "*".into(),
                payload,
            )
            .await
        {
            return Err(ProducerError::RedisError(e.to_string()));
        }

        Ok(())
    }
}
