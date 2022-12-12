use std::sync::Arc;

use chrono::DateTime;
use tokio::sync::{mpsc, oneshot};

use crate::{
    features::livestream::definition::DEVICE_LIVESTREAM_QUEUE_NAME,
    queue::{Consumer, ConsumerError},
    shutdown::Shutdown,
};

use super::{
    definition::{LivestreamDeviceMap, LivestreamMessagePayload, LivestreamSessionMap},
    service::LivestreamServiceInterface,
    socket::LivestreamMessage,
};

pub async fn run(
    mut shutdown: Shutdown,
    _sender: mpsc::Sender<()>,
    redis: deadpool_redis::Pool,
    livestream_service: Arc<dyn LivestreamServiceInterface>,
    sessions: LivestreamSessionMap,
    devices: LivestreamDeviceMap,
) {
    let (tx, mut rx) = mpsc::channel::<oneshot::Sender<bool>>(32);
    let tx_2 = tx.clone();

    let listener = actix_web::rt::spawn(async move {
        let mut consumer = Consumer::new(redis, DEVICE_LIVESTREAM_QUEUE_NAME.to_string());

        println!("Livestream consumer has started");

        loop {
            if let Ok(resp) = rx.try_recv() {
                let _ = resp.send(true);
                break;
            }

            let pending_message_id = match consumer.get_pending_message_id().await {
                Ok(id) => id,
                Err(_) => continue,
            };

            let livestream_service = livestream_service.clone();

            let sessions = sessions.clone();
            let devices = devices.clone();

            let message_id = if let Some(message_id) = pending_message_id {
                handle_pending_message(
                    &mut consumer,
                    livestream_service,
                    sessions,
                    devices,
                    message_id,
                )
                .await
            } else {
                #[allow(unused_assignments)]
                let mut message_id: Option<String> = None;

                tokio::select! {
                    _response = rx.recv() => {
                        break;
                    },
                    data = consumer.consume_raw() => {
                        message_id = handle_upcoming_message(data, livestream_service, sessions, devices).await;
                    },
                }

                message_id
            };

            if let Some(id) = message_id {
                if let Err(_) = consumer.ack(id).await {
                    continue;
                }
            }
        }
    });

    let shutdown_listener = actix_web::rt::spawn(async move {
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
        println!("Livestream consumer finished shutting down");
    });

    tokio::try_join!(listener, shutdown_listener).unwrap();
}

async fn handle_pending_message(
    consumer: &mut Consumer,
    livestream_service: Arc<dyn LivestreamServiceInterface>,
    sessions: LivestreamSessionMap,
    devices: LivestreamDeviceMap,
    message_id: String,
) -> Option<String> {
    let data = match consumer
        .read_raw_by_message_id(message_id.to_string())
        .await
    {
        Ok(res) => res,
        Err(_) => return None,
    };

    if data.len() == 0 {
        return Some(message_id.to_string());
    }

    let (message_id, payload) = &data[0];

    let livestream_message = match parse_livestream_message(payload.to_string()) {
        Some(msg) => msg,
        None => return Some(message_id.to_string()),
    };

    if let Err(_) = livestream_service.insert(livestream_message.clone()).await {
        return Some(message_id.to_string());
    };

    publish(sessions, devices, livestream_message);

    return Some(message_id.to_string());
}

async fn handle_upcoming_message(
    result: Result<Vec<(String, String)>, ConsumerError>,
    livestream_service: Arc<dyn LivestreamServiceInterface>,
    sessions: LivestreamSessionMap,
    devices: LivestreamDeviceMap,
) -> Option<String> {
    let data = match result {
        Ok(res) => res,
        Err(_) => return None,
    };

    if data.len() == 0 {
        return None;
    }

    let (message_id, payload) = &data[0];

    let livestream_message = match parse_livestream_message(payload.to_string()) {
        Some(msg) => msg,
        None => return Some(message_id.to_string()),
    };

    if let Err(_) = livestream_service.insert(livestream_message.clone()).await {
        return Some(message_id.to_string());
    };

    publish(sessions, devices, livestream_message);

    return Some(message_id.to_string());
}

fn parse_livestream_message(message: String) -> Option<LivestreamMessagePayload> {
    let splitted: Vec<String> = message.split(" ").map(|s| s.to_string()).collect();
    if splitted.len() != 3 {
        return None;
    }

    let timestamp = match DateTime::parse_from_rfc3339(&splitted[0]) {
        Ok(date) => date,
        Err(_) => return None,
    };

    let device_id = match splitted[1].parse::<i32>() {
        Ok(id) => id,
        Err(_) => return None,
    };

    let num_of_faces = match splitted[2].parse::<i32>() {
        Ok(num) => num,
        Err(_) => return None,
    };

    Some(LivestreamMessagePayload {
        timestamp,
        device_id,
        num_of_faces,
    })
}

fn publish(
    sessions: LivestreamSessionMap,
    devices: LivestreamDeviceMap,
    payload: LivestreamMessagePayload,
) {
    let devices = devices.lock().unwrap();
    let sessions = sessions.lock().unwrap();

    let device_id = payload.device_id;
    let payload = serde_json::to_string(&payload).unwrap();

    if let Some(device_sessions) = devices.get(&device_id) {
        for session_id in device_sessions {
            if let Some(session_addr) = sessions.get(&session_id) {
                session_addr.do_send(LivestreamMessage(payload.clone()))
            }
        }
    }
}
