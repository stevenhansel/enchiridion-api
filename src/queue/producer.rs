use std::collections::BTreeMap;

use redis::AsyncCommands;

pub enum ProducerError {
    RedisError(String),
}

pub struct Producer {
    pub client: deadpool_redis::Pool,
    pub queue_name: String,
    pub group_name: &'static str,
}

impl Producer {
    pub fn new(client: deadpool_redis::Pool, queue_name: String) -> Producer {
        Producer {
            client,
            queue_name,
            group_name: "main-group",
        }
    }

    async fn initialize_consumer_group(&self) -> Result<(), ProducerError> {
        let mut redis = self
            .client
            .get()
            .await
            .expect("Cannot get redis connection");

        if let Err(e) = redis
            .xgroup_create_mkstream::<String, String, String, ()>(
                self.queue_name.clone(),
                self.group_name.to_string(),
                "$".into(),
            )
            .await
        {
            if let Some(code) = e.code() {
                if code != "BUSYGROUP" {
                    return Err(ProducerError::RedisError(e.to_string()));
                }
            }
        };

        Ok(())
    }

    pub async fn push(&self, payload: BTreeMap<String, String>) -> Result<(), ProducerError> {
        self.initialize_consumer_group().await?;

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
