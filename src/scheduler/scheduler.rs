use std::{str::FromStr, sync::Arc};

use chrono::Utc;
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
    let announcement_service_1 = announcement_service.clone();
    let announcement_service_2 = announcement_service.clone();
    let announcement_service_3 = announcement_service.clone();

    let waiting_for_approval_handler = tokio::spawn(async move {
        announcement_service_1
            .handle_waiting_for_approval_announcements()
            .await
    });
    let waiting_for_sync_handler = tokio::spawn(async move {
        announcement_service_2
            .handle_waiting_for_sync_announcements()
            .await
    });
    let active_handler =
        tokio::spawn(async move { announcement_service_3.handle_active_announcements().await });

    if let Ok(result) = waiting_for_approval_handler.await {
        if let Err(e) = result {
            return Err(e);
        }
    } else {
        return Err(HandleScheduledAnnouncementsError::BrokenThread);
    }

    if let Ok(result) = waiting_for_sync_handler.await {
        if let Err(e) = result {
            return Err(e);
        }
    } else {
        return Err(HandleScheduledAnnouncementsError::BrokenThread);
    }

    if let Ok(result) = active_handler.await {
        if let Err(e) = result {
            return Err(e);
        }
    } else {
        return Err(HandleScheduledAnnouncementsError::BrokenThread);
    }

    Ok(())
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

        let schedule = Schedule::from_str("0 0 */12 * * *").unwrap();
        let mut last_tick = Utc::now();

        loop {
            if let Ok(resp) = rx.try_recv() {
                let _ = resp.send(true);
                break;
            }

            let now = Utc::now();

            if let Some(event) = schedule.after(&last_tick).take(1).next() {
                if now > event {
                    println!(
                        "[info] Announcement scheduler started processing at {}",
                        now
                    );

                    if let Err(e) = execute_announcement_scheduler(announcement_service.clone()).await {
                        eprintln!("[error] Something went wrong when executing the announcement scheduler: {}", e);
                    }

                    println!("[info] Announcement scheduler finished processing");
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
