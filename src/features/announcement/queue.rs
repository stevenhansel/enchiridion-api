use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::Serialize;

use crate::queue::Producer;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnouncementSyncAction {
    Create,
    Delete,
    Resync,
}

pub enum AnnouncementQueueError {
    PayloadSerializationError(String),
    InternalServerError,
}

impl std::fmt::Display for AnnouncementQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnouncementQueueError::PayloadSerializationError(message) => write!(f, "{}", message),
            AnnouncementQueueError::InternalServerError => write!(f, "Internal Server Error"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceSynchronizationParams<T> {
    action: AnnouncementSyncAction,
    data: T,
}

#[async_trait]
pub trait AnnouncementQueueInterface {
    async fn create(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError>;
    async fn delete(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError>;
    async fn resync(
        &self,
        device_id: i32,
        announcement_ids: Vec<i32>,
    ) -> Result<(), AnnouncementQueueError>;
}

pub struct AnnouncementQueue {
    _redis: deadpool_redis::Pool,
}

impl AnnouncementQueue {
    pub fn new(_redis: deadpool_redis::Pool) -> Self {
        AnnouncementQueue { _redis }
    }

    pub fn create_producer(&self, device_id: i32) -> Producer {
        Producer::new(self._redis.clone(), format!("device-queue-{}", device_id))
    }

    pub fn create_payload_map(&self, data: &String) -> BTreeMap<String, String> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.insert("data".into(), data.clone());

        map
    }
}

#[async_trait]
impl AnnouncementQueueInterface for AnnouncementQueue {
    async fn create(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError> {
        let params: DeviceSynchronizationParams<i32> = DeviceSynchronizationParams {
            action: AnnouncementSyncAction::Create,
            data: announcement_id,
        };
        let payload = match serde_json::to_string(&params) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(AnnouncementQueueError::PayloadSerializationError(
                    e.to_string(),
                ))
            }
        };

        for device_id in device_ids {
            let producer = self.create_producer(device_id);
            if let Err(_) = producer.push(self.create_payload_map(&payload)).await {
                return Err(AnnouncementQueueError::InternalServerError);
            };
        }

        Ok(())
    }

    async fn delete(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError> {
        let params: DeviceSynchronizationParams<i32> = DeviceSynchronizationParams {
            action: AnnouncementSyncAction::Delete,
            data: announcement_id,
        };
        let payload = match serde_json::to_string(&params) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(AnnouncementQueueError::PayloadSerializationError(
                    e.to_string(),
                ))
            }
        };

        for device_id in device_ids {
            let producer = self.create_producer(device_id);
            if let Err(_) = producer.push(self.create_payload_map(&payload)).await {
                return Err(AnnouncementQueueError::InternalServerError);
            };
        }

        Ok(())
    }

    async fn resync(
        &self,
        device_id: i32,
        announcement_ids: Vec<i32>,
    ) -> Result<(), AnnouncementQueueError> {
        let params: DeviceSynchronizationParams<Vec<i32>> = DeviceSynchronizationParams {
            action: AnnouncementSyncAction::Resync,
            data: announcement_ids,
        };
        let payload = match serde_json::to_string(&params) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(AnnouncementQueueError::PayloadSerializationError(
                    e.to_string(),
                ))
            }
        };

        let producer = self.create_producer(device_id);
        if let Err(_) = producer.push(self.create_payload_map(&payload)).await {
            return Err(AnnouncementQueueError::InternalServerError);
        };

        Ok(())
    }
}
