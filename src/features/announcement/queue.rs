use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use serde::Serialize;

use crate::queue::Producer;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnouncementSyncAction {
    Create,
    Delete,
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
#[serde(rename_all = "camelCase")]
pub struct SyncCreateAnnouncementActionParams {
    action: AnnouncementSyncAction,
    announcement_id: i32,
}

#[async_trait]
pub trait AnnouncementQueueInterface {
    fn synchronize_create_announcement_action_to_devices(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError>;
    fn synchronize_delete_announcement_action_to_devices(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError>;
}

pub struct AnnouncementQueue {
    _redis: Arc<Mutex<redis::Connection>>,
}

impl AnnouncementQueue {
    pub fn new(_redis: Arc<Mutex<redis::Connection>>) -> Self {
        AnnouncementQueue { _redis }
    }

    pub fn queue_name_builder(&self, device_id: i32) -> String {
        format!("device-queue-{}", device_id)
    }
}

#[async_trait]
impl AnnouncementQueueInterface for AnnouncementQueue {
    fn synchronize_create_announcement_action_to_devices(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError> {
        let payload = match serde_json::to_string(&SyncCreateAnnouncementActionParams {
            action: AnnouncementSyncAction::Create,
            announcement_id,
        }) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(AnnouncementQueueError::PayloadSerializationError(
                    e.to_string(),
                ))
            }
        };

        for id in device_ids {
            let mut map: BTreeMap<String, String> = BTreeMap::new();
            map.insert(String::from("data"), payload.clone());

            let producer = Producer::new(self._redis.clone(), self.queue_name_builder(id));
            if let Err(_) = producer.push(map) {
                return Err(AnnouncementQueueError::InternalServerError);
            };
        }

        Ok(())
    }

    fn synchronize_delete_announcement_action_to_devices(
        &self,
        device_ids: Vec<i32>,
        announcement_id: i32,
    ) -> Result<(), AnnouncementQueueError> {
        let payload = match serde_json::to_string(&SyncCreateAnnouncementActionParams {
            action: AnnouncementSyncAction::Delete,
            announcement_id,
        }) {
            Ok(payload) => payload,
            Err(e) => {
                return Err(AnnouncementQueueError::PayloadSerializationError(
                    e.to_string(),
                ))
            }
        };

        for id in device_ids {
            let mut map: BTreeMap<String, String> = BTreeMap::new();
            map.insert(String::from("data"), payload.clone());

            let producer = Producer::new(self._redis.clone(), self.queue_name_builder(id));
            if let Err(_) = producer.push(map) {
                return Err(AnnouncementQueueError::InternalServerError);
            };
        }

        Ok(())
    }
}
