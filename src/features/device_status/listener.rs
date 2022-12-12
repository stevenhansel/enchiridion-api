use actix::Recipient;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use deadpool_redis::redis::cmd;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::shutdown::Shutdown;

use super::{
    definition::{DeviceStatus, DEVICE_STATUS_REDIS_KEY, SLEEP_DURATION, TIMEOUT_DURATION_SECS},
    socket::StatusMessage,
};

pub async fn run(
    mut shutdown: Shutdown,
    _sender: mpsc::Sender<()>,
    redis: deadpool_redis::Pool,
    sessions: Arc<Mutex<HashMap<usize, Recipient<StatusMessage>>>>,
    devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
) {
    let (tx, mut rx) = mpsc::channel::<oneshot::Sender<bool>>(32);
    let tx_2 = tx.clone();

    let listener = tokio::spawn(async move {
        let mut conn = redis.get().await.expect("Cannot get redis connection");
        let mut map: HashMap<i32, DateTime<FixedOffset>> = HashMap::new();

        println!("Device status listener has started");

        loop {
            if let Ok(resp) = rx.try_recv() {
                let _ = resp.send(true);
                break;
            }

            sleep(SLEEP_DURATION).await;

            let sessions = sessions.clone();
            let devices = devices.clone();

            let result = match cmd("HGETALL")
                .arg(&[DEVICE_STATUS_REDIS_KEY])
                .query_async::<_, HashMap<i32, String>>(&mut conn)
                .await
            {
                Ok(result) => result,
                Err(_) => continue,
            };

            let mut connected_device_ids: Vec<i32> = Vec::new();
            let mut disconnected_device_ids: Vec<i32> = Vec::new();
            let mut unregistered_device_ids: Vec<i32> = Vec::new();

            for (key, value) in result {
                let parsed_date = match DateTime::parse_from_rfc3339(value.as_str()) {
                    Ok(date) => Some(date),
                    Err(_) => None,
                };

                if let Some(date) = parsed_date {
                    if let Some(existing_value) = map.get_mut(&key) {
                        *existing_value = date;
                    } else {
                        map.insert(key, date);
                    }
                } else {
                    unregistered_device_ids.push(key);
                }
            }

            let now = Utc::now();

            for (key, value) in &map {
                if *value + Duration::seconds(TIMEOUT_DURATION_SECS) < now {
                    disconnected_device_ids.push(*key);
                } else {
                    connected_device_ids.push(*key);
                }
            }

            publish(
                sessions.clone(),
                devices.clone(),
                &connected_device_ids,
                DeviceStatus::Connected,
            );
            publish(
                sessions.clone(),
                devices.clone(),
                &disconnected_device_ids,
                DeviceStatus::Disconnected,
            );
            publish(
                sessions,
                devices,
                &unregistered_device_ids,
                DeviceStatus::Unregistered,
            );
        }
    });

    let shutdown_listener = tokio::spawn(async move {
        let _ = shutdown.recv().await;

        let (resp_tx, resp_rx) = oneshot::channel::<bool>();
        if let Err(e) = tx_2.send(resp_tx).await {
            eprintln!(
                "Something went wrong when sending shutdown signal: {}",
                e.to_string()
            );
            return;
        }

        let _ = resp_rx.await;
        println!("Device status listener finished shutting down");
    });

    tokio::try_join!(listener, shutdown_listener).unwrap();
}

pub fn publish(
    sessions: Arc<Mutex<HashMap<usize, Recipient<StatusMessage>>>>,
    devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
    device_ids: &Vec<i32>,
    status: DeviceStatus,
) {
    let devices = devices.lock().unwrap();
    let sessions = sessions.lock().unwrap();

    for device_id in device_ids {
        if let Some(device_sessions) = devices.get(device_id) {
            for session_id in device_sessions {
                if let Some(session_addr) = sessions.get(&session_id) {
                    session_addr.do_send(StatusMessage(status.to_string()))
                }
            }
        }
    }
}
