use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use log::info;
use crate::db::Database;
use crate::shared::{BufferManager, LifecycleManager};

pub struct AndroidBackgroundService {
    db: Arc<Database>,
    shutdown_rx: watch::Receiver<bool>,
}

impl AndroidBackgroundService {
    pub fn new(db: Arc<Database>, shutdown_rx: watch::Receiver<bool>) -> Self {
        Self { db, shutdown_rx }
    }

    pub async fn run(&self) {
        info!("AndroidBackgroundService started");

        // Create buffer manager
        let buffer_manager = BufferManager::new(self.db.clone());

        // Initial buffer check
        let _ = buffer_manager.replenish_buffer_if_needed().await;

        loop {
            // Check for shutdown signal
            let mut rx = self.shutdown_rx.clone();
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => {
                    // 30 minutes elapsed, check buffer
                    info!("Android background service: checking buffer after 30 minutes");

                    // Check if app is in background mode
                    if LifecycleManager::is_background_mode() {
                        let result = buffer_manager.replenish_buffer_if_needed().await;
                        match result {
                            Ok(downloaded) => {
                                if downloaded > 0 {
                                    info!("Downloaded {} items in background", downloaded);
                                }
                            }
                            Err(e) => {
                                log::warn!("Background download failed: {}", e);
                                // Will retry in 30 minutes
                            }
                        }
                    } else {
                        info!("App in foreground, skipping background download");
                    }
                }
                _ = rx.changed() => {
                    if *rx.borrow() {
                        info!("AndroidBackgroundService shutting down");
                        break;
                    }
                }
            }
        }
    }
}
