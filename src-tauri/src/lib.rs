mod commands;
mod crawler;
mod db;
mod notifications;
mod summary;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let database =
                Arc::new(db::Database::new(app_dir).expect("failed to initialize database"));

            app.manage(database.clone());

            let crawler_db = database.clone();
            let crawler_shutdown = shutdown_rx.clone();
            let notif_db = database.clone();
            let notif_shutdown = shutdown_rx.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let scheduler =
                    crawler::CrawlScheduler::new(crawler_db, crawler_shutdown);
                scheduler.run().await;
            });

            tauri::async_runtime::spawn(async move {
                let engine =
                    notifications::NotificationEngine::new(notif_db, notif_shutdown);
                engine.run(app_handle).await;
            });

            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let _ = shutdown_tx.send(true);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today_items,
            commands::get_items_by_category,
            commands::get_today_stats,
            commands::get_daily_summary,
            commands::toggle_save_item,
            commands::mark_item_seen,
            commands::consume_pending_items,
            commands::open_url,
            commands::get_throttle_level,
            commands::set_throttle_level,
            commands::get_consumption_threads,
            commands::set_consumption_threads,
            commands::get_pending_count,
            commands::prune_old_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running cazzmachine");
}
