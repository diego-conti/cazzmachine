pub mod background_service;

use crate::shared::LifecycleManager;

#[tauri::command]
pub fn on_android_app_background() {
    LifecycleManager::on_background();
    log::info!("Android app entered background");
}

#[tauri::command]
pub fn on_android_app_foreground() {
    LifecycleManager::on_foreground();
    log::info!("Android app entered foreground");
}

#[tauri::command]
pub fn on_android_resume(db: State<'_, Arc<Database>>) -> Result<f64, String> {
    // Calculate elapsed time since last active
    let last_active = db.get_last_active_timestamp()
        .map_err(|e| e.to_string())?;

    let elapsed_minutes = LifecycleManager::get_elapsed_minutes(last_active);

    // Update last active timestamp
    let now = chrono::Utc::now();
    db.set_last_active_timestamp(now)
        .map_err(|e| e.to_string())?;

    log::info!("Android resume: elapsed {:.2} minutes", elapsed_minutes);

    Ok(elapsed_minutes)
}
