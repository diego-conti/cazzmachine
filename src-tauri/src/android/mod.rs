pub mod background_service;

use std::sync::Arc;
use tauri::State;
use crate::db::Database;
use crate::shared::lifecycle_manager::LifecycleManager;

#[tauri::command]
pub fn on_android_app_background(db: State<'_, Arc<Database>>) -> Result<(), String> {
    LifecycleManager::on_background();
    db.log_diagnostic_event("android_lifecycle", "info", "Android app entered background", None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn on_android_app_foreground(db: State<'_, Arc<Database>>) -> Result<(), String> {
    LifecycleManager::on_foreground();
    db.log_diagnostic_event("android_lifecycle", "info", "Android app entered foreground", None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn on_android_resume(db: State<'_, Arc<Database>>) -> Result<f64, String> {
    // Calculate elapsed time since last active
    let last_active = db.get_last_active_timestamp()
        .map_err(|e: rusqlite::Error| e.to_string())?;

    let elapsed_minutes = LifecycleManager::get_elapsed_minutes(last_active);

    // Update last active timestamp
    let now = chrono::Utc::now();
    db.set_last_active_timestamp(now)
        .map_err(|e: rusqlite::Error| e.to_string())?;

    db.log_diagnostic_event("android_lifecycle", "info", &format!("Android resume: elapsed {:.2} minutes", elapsed_minutes), None, None)
        .map_err(|e| e.to_string())?;

    Ok(elapsed_minutes)
}
