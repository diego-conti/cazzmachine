pub mod templates;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

use crate::commands::THROTTLE_LEVEL;
use crate::db::Database;
use templates::generate_teaser;

pub struct NotificationEngine {
    db: Arc<Database>,
    shutdown_rx: watch::Receiver<bool>,
}

impl NotificationEngine {
    pub fn new(db: Arc<Database>, shutdown_rx: watch::Receiver<bool>) -> Self {
        Self { db, shutdown_rx }
    }

    fn get_notify_interval(&self) -> Duration {
        let level = THROTTLE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        // Formulas for linear active% from 2% to 91%:
        // S(level) = 1 + 4 × (level-1)/8          (scroll: 1→5 min)
        // A(level) = 0.02 + 0.89 × (level-1)/8   (active%: 2%→91%)
        // W(level) = S × ((1/A) - 1)             (standby: convex curve)
        let level_f = level as f64;
        let scroll_minutes = 1.0 + 4.0 * ((level_f - 1.0) / 8.0);
        let active_pct = 0.02 + 0.89 * ((level_f - 1.0) / 8.0);
        let standby_minutes = scroll_minutes * ((1.0 / active_pct) - 1.0);
        Duration::from_secs((standby_minutes * 60.0) as u64)
    }

    pub async fn run(&self, app_handle: tauri::AppHandle) {
        log::info!("NotificationEngine started");
        let initial_interval = self.get_notify_interval();
        log::info!("Notification interval: {} seconds", initial_interval.as_secs());

        tokio::time::sleep(Duration::from_secs(30)).await;

        loop {
            self.send_teaser(&app_handle).await;

            let interval = self.get_notify_interval();
            log::info!("Next notification in {} seconds", interval.as_secs());

            let mut rx = self.shutdown_rx.clone();
            tokio::select! {
                _ = tokio::time::sleep(interval) => {},
                _ = rx.changed() => {
                    if *rx.borrow() {
                        log::info!("NotificationEngine shutting down");
                        break;
                    }
                }
            }
        }
    }

    async fn send_teaser(&self, app_handle: &tauri::AppHandle) {
        let stats = match self.db.get_today_stats() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to get stats for notification: {}", e);
                return;
            }
        };

        let latest_item = self.db.get_latest_unseen_item().ok().flatten();
        let message = generate_teaser(&stats, latest_item.as_ref());

        // Emit event to trigger frontend doomscrolling cycle (no OS notification)
        if let Err(e) = tauri::Emitter::emit(app_handle, "cazz-notification", &message) {
            log::warn!("Failed to emit notification event: {}", e);
        }

        if let Some(ref item) = latest_item {
            let _ = self.db.mark_item_seen(&item.id);
        }
        let _ = self.db.log_notification(&message, latest_item.as_ref().map(|i| i.id.as_str()));
    }
}
