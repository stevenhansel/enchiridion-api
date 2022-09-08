use std::{str::FromStr, sync::Arc};

use chrono::{NaiveTime, TimeZone, Utc};
use chrono_tz::{Asia::Jakarta, Tz};
use cron::Schedule;
use tokio::{
    sync::{mpsc, oneshot},
    time::{sleep, Duration},
};

use crate::{
    features::{AnnouncementServiceInterface, HandleScheduledAnnouncementsError},
    shutdown::Shutdown,
};

pub async fn execute_announcement_scheduler(
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
) -> Result<(), HandleScheduledAnnouncementsError> {
    let now =
        chrono::DateTime::from_utc(chrono::Utc::today().naive_utc().and_hms(0, 0, 0), chrono::Utc);

    let announcement_service_1 = announcement_service.clone();
    let announcement_service_2 = announcement_service.clone();
    let announcement_service_3 = announcement_service.clone();

    let waiting_for_approval_handler = tokio::spawn(async move {
        announcement_service_1
            .handle_waiting_for_approval_announcements(now)
            .await
    });

    let waiting_for_sync_handler = tokio::spawn(async move {
        announcement_service_2
            .handle_waiting_for_sync_announcements(now)
            .await
    });
    let active_handler =
        tokio::spawn(async move { announcement_service_3.handle_active_announcements().await });

    match tokio::try_join!(
        waiting_for_approval_handler,
        waiting_for_sync_handler,
        active_handler
    ) {
        Ok(_result) => Ok(()),
        Err(_) => Err(HandleScheduledAnnouncementsError::InternalServerError),
    }
}

pub async fn run(
    mut shutdown: Shutdown,
    _sender: mpsc::Sender<()>,
    announcement_service: Arc<dyn AnnouncementServiceInterface + Send + Sync + 'static>,
) {
    // TODO: refactor scheduler in the future so can have more than one cron

    let (tx, mut rx) = mpsc::channel::<oneshot::Sender<bool>>(32);
    let tx_2 = tx.clone();

    let cron = tokio::spawn(async move {
        println!("[info] Announcement Scheduler is starting");

        let schedule = Schedule::from_str("0 0 0 * * *").unwrap();

        let mut last_tick: Option<chrono::DateTime<Tz>> = None;
        loop {
            if let Ok(resp) = rx.try_recv() {
                let _ = resp.send(true);
                break;
            }

            sleep(Duration::from_millis(250)).await;

            if last_tick == None {
                let today_utc = (Utc::today() - chrono::Duration::days(1))
                    .and_time(NaiveTime::from_hms(17, 0, 0))
                    .unwrap();

                last_tick = Some(Jakarta.from_utc_datetime(&today_utc.naive_utc()));

                continue;
            }

            let utc_now = Utc::now().naive_utc();
            let now = Jakarta.from_utc_datetime(&utc_now);

            if let Some(event) = schedule.after(&last_tick.unwrap()).take(1).next() {
                if event > now {
                    continue;
                }

                println!(
                    "[info] Announcement scheduler started processing at {}",
                    now
                );

                if let Err(e) = execute_announcement_scheduler(announcement_service.clone()).await {
                    eprintln!("[error] Something went wrong when executing the announcement scheduler: {}", e);
                }

                println!("[info] Announcement scheduler finished processing");
            }

            last_tick = Some(now);
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

    tokio::try_join!(cron, shutdown_listener).unwrap();
}
