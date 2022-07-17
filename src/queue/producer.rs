use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use redis::Commands;

pub enum ProducerError {
    RedisError(String),
}

pub struct Producer {
    client: Arc<Mutex<redis::Connection>>,
    queue_name: String,
}

impl Producer {
    pub fn new(client: Arc<Mutex<redis::Connection>>, queue_name: String) -> Producer {
        Producer { client, queue_name }
    }

    pub fn push(&self, payload: BTreeMap<String, String>) -> Result<(), ProducerError> {
        let mut redis = self
            .client
            .lock()
            .expect("Cannot get redis client connection");

        if let Err(e) = redis.xadd_map::<String, String, BTreeMap<String, String>, ()>(
            self.queue_name.clone(),
            "*".into(),
            payload,
        ) {
            return Err(ProducerError::RedisError(e.to_string()));
        }

        Ok(())
    }
}
