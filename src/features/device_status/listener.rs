use chrono::{DateTime, Duration, FixedOffset, Utc};
use deadpool_redis::redis::cmd;
use std::collections::HashMap;

use tokio::time::sleep;

const DEVICE_STATUS_REDIS_KEY: &'static str = "device_status";

const SLEEP_DURATION: std::time::Duration = std::time::Duration::from_secs(1);
const TIMEOUT_DURATION_SECS: i64 = 3;

pub async fn run(redis: deadpool_redis::Pool) {
    let mut conn = redis.get().await.expect("Cannot get redis connection");
    let mut map: HashMap<i32, DateTime<FixedOffset>> = HashMap::new();

    loop {
        sleep(SLEEP_DURATION).await;

        let result = match cmd("HGETALL")
            .arg(&[DEVICE_STATUS_REDIS_KEY])
            .query_async::<_, HashMap<i32, String>>(&mut conn)
            .await
        {
            Ok(result) => result,
            Err(_) => continue,
        };

        for (key, value) in result {
            let date = match DateTime::parse_from_rfc3339(value.as_str()) {
                Ok(date) => date,
                Err(_) => {
                    continue
                },
            };

            if let Some(existing_value) = map.get_mut(&key) {
                *existing_value = date;
            } else {
                map.insert(key, date);
            }
        }

        let now = Utc::now();
        for (key, value) in &map {
            if *value + Duration::seconds(TIMEOUT_DURATION_SECS) < now {
                println!("device id {} is timed out", key);
            } else {
                println!("device id {} is connected", key)
            }
        }
    }
}
