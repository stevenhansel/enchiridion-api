use std::str::FromStr;

use chrono::{TimeZone, Utc};
use chrono_tz::Asia::Jakarta;
use cron::Schedule;
use tokio::{
    sync::{mpsc, oneshot},
    time::{sleep, Duration},
};

use crate::shutdown::Shutdown;

pub async fn run(mut shutdown: Shutdown, _sender: mpsc::Sender<()>) {
    let (tx, mut rx) = mpsc::channel::<oneshot::Sender<bool>>(32);
    let tx_2 = tx.clone();

    let cron = tokio::spawn(async move {
        println!("[info] Announcement Scheduler is starting");

        let schedule = Schedule::from_str("0 0 0 * * *").unwrap();
        let now = Utc::now().naive_utc();
        let mut last_tick = Jakarta.from_utc_datetime(&now);

        loop {
            if let Ok(resp) = rx.try_recv() {
                let _ = resp.send(true);
                break;
            }

            let now = Utc::now().naive_utc();
            let now = Jakarta.from_utc_datetime(&now);

            if let Some(event) = schedule.after(&last_tick).take(1).next() {
                if now > event {
                    println!("cron start processing");

                    sleep(Duration::from_millis(5000)).await;

                    println!("cron finished processing");
                }
            }

            last_tick = now;
            sleep(Duration::from_millis(1000)).await;
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
        println!("[info] Announcement Scheduler finished shutting down");
    });

    cron.await.unwrap();
    shutdown_listener.await.unwrap();
}
