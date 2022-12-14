use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::Recipient;
use serde::{Deserialize, Serialize};

use super::socket::LivestreamMessage;

pub const DEVICE_LIVESTREAM_QUEUE_NAME: &str = "device_livestream";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LivestreamMessagePayload {
    pub timestamp: chrono::DateTime<chrono::FixedOffset>,
    pub device_id: i32,
    pub num_of_faces: i32,
}

pub type LivestreamSessionMap = Arc<Mutex<HashMap<usize, Recipient<LivestreamMessage>>>>;
pub type LivestreamDeviceMap = Arc<Mutex<HashMap<i32, HashSet<usize>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LivestreamQueryAction {
    Average,
    Max,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LivestreamInterval {
    Minute,
    Hour,
    Day,
}

impl std::fmt::Display for LivestreamInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LivestreamInterval::Minute => write!(f, "1 minute"),
            LivestreamInterval::Hour => write!(f, "1 hour"),
            LivestreamInterval::Day => write!(f, "1 day"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LivestreamRange {
    Hour,
    Day,
    Week,
}

impl std::fmt::Display for LivestreamRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LivestreamRange::Hour => write!(f, "1 hour"),
            LivestreamRange::Day => write!(f, "1 day"),
            LivestreamRange::Week => write!(f, "7 days"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLivestreamQueryResult {
    pub action: LivestreamQueryAction,
    pub interval: LivestreamInterval,
    pub range: LivestreamRange,
    pub contents: Vec<DeviceLivestreamContent>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLivestreamContent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}
