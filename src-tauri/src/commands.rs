use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tauri::State;

use crate::db::models::{ConsumeResult, CrawlItem, DayStats, DaySummary};
use crate::db::Database;
use crate::summary;

pub static THROTTLE_LEVEL: AtomicU8 = AtomicU8::new(5);
pub static THREAD_COUNT: AtomicU8 = AtomicU8::new(1);

#[tauri::command]
pub fn get_throttle_level() -> u8 {
    THROTTLE_LEVEL.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_throttle_level(level: u8) {
    let level = level.clamp(1, 9);
    THROTTLE_LEVEL.store(level, Ordering::Relaxed);
    log::info!("Throttle level set to {}", level);
}

#[tauri::command]
pub fn get_consumption_threads() -> u8 {
    THREAD_COUNT.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_consumption_threads(count: u8) {
    let count = count.clamp(1, 8);
    THREAD_COUNT.store(count, Ordering::Relaxed);
    log::info!("Thread count set to {}", count);
}

#[tauri::command]
pub fn get_today_items(db: State<'_, Arc<Database>>) -> Result<Vec<CrawlItem>, String> {
    db.get_items_for_today().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_items_by_category(
    db: State<'_, Arc<Database>>,
    category: String,
) -> Result<Vec<CrawlItem>, String> {
    db.get_items_by_category(&category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_today_stats(db: State<'_, Arc<Database>>) -> Result<DayStats, String> {
    db.get_today_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_summary(db: State<'_, Arc<Database>>) -> Result<DaySummary, String> {
    summary::generate_daily_summary(&db)
}

#[tauri::command]
pub fn toggle_save_item(db: State<'_, Arc<Database>>, item_id: String) -> Result<bool, String> {
    db.toggle_item_saved(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_item_seen(db: State<'_, Arc<Database>>, item_id: String) -> Result<(), String> {
    db.mark_item_seen(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn consume_pending_items(
    db: State<'_, Arc<Database>>,
    budget_minutes: f64,
) -> Result<ConsumeResult, String> {
    db.consume_pending_items(budget_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if cfg!(target_os = "android") {
        open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
    } else {
        open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
    }
}

#[tauri::command]
pub fn prune_old_items(db: State<'_, Arc<Database>>) -> Result<(i64, i64), String> {
    db.prune_old_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pending_count(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    db.get_pending_count().map_err(|e| e.to_string())
}
