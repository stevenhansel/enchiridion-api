use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::Recipient;
use serde::{Deserialize, Serialize};

use super::socket::LivestreamMessage;

pub const DEVICE_LIVESTREAM_QUEUE_NAME: &'static str = "device_livestream";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivestreamMessagePayload {
    pub timestamp: chrono::DateTime<chrono::FixedOffset>,
    pub device_id: i32,
    pub num_of_faces: i32,
}

pub type LivestreamSessionMap = Arc<Mutex<HashMap<usize, Recipient<LivestreamMessage>>>>;
pub type LivestreamDeviceMap = Arc<Mutex<HashMap<i32, HashSet<usize>>>>;
