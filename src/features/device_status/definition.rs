pub const DEVICE_STATUS_REDIS_KEY: &'static str = "device_status";

pub const SLEEP_DURATION: std::time::Duration = std::time::Duration::from_secs(1);
pub const TIMEOUT_DURATION_SECS: i64 = 3;

#[derive(Debug, PartialEq)]
pub enum DeviceStatus {
    Connected,
    Disconnected,
    Unregistered,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DeviceStatus::Connected => write!(f, "Connected"),
            DeviceStatus::Disconnected => write!(f, "Disconnected"),
            DeviceStatus::Unregistered => write!(f, "Unregistered"),
        }
    }
}

