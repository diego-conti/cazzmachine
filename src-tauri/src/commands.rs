use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tauri::State;
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::db::models::{
    ClearDiagnosticsResult, ConsumeResult, CrawlItem, DayStats, DaySummary, DiagnosticLog,
    DiagnosticSummary, ProviderStatus,
};
use crate::db::Database;
use crate::summary;
use crate::crawler;
use crate::crawler::provider::ContentProvider;

pub static THROTTLE_LEVEL: AtomicU8 = AtomicU8::new(5);
pub static THREAD_COUNT: AtomicU8 = AtomicU8::new(1);

#[tauri::command]
pub fn get_throttle_level() -> u8 {
    THROTTLE_LEVEL.load(Ordering::Relaxed)
}

#[tauri::command]
pub async fn set_throttle_level(db: State<'_, Arc<Database>>, level: u8) -> Result<(), String> {
    let level = level.clamp(1, 9);
    THROTTLE_LEVEL.store(level, Ordering::Relaxed);
    db.log_diagnostic_event("setting_change", "info", &format!("Throttle level set to {}", level), None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_consumption_threads() -> u8 {
    THREAD_COUNT.load(Ordering::Relaxed)
}

#[tauri::command]
pub async fn set_consumption_threads(db: State<'_, Arc<Database>>, count: u8) -> Result<(), String> {
    let count = count.clamp(1, 8);
    THREAD_COUNT.store(count, Ordering::Relaxed);
    db.log_diagnostic_event("setting_change", "info", &format!("Thread count set to {}", count), None, None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_today_items(db: State<'_, Arc<Database>>) -> Result<Vec<CrawlItem>, String> {
    db.get_items_for_today().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_items_by_category(
    db: State<'_, Arc<Database>>,
    category: String,
) -> Result<Vec<CrawlItem>, String> {
    db.get_items_by_category(&category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_today_stats(db: State<'_, Arc<Database>>) -> Result<DayStats, String> {
    db.get_today_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daily_summary(db: State<'_, Arc<Database>>) -> Result<DaySummary, String> {
    summary::generate_daily_summary(&db)
}

#[tauri::command]
pub async fn toggle_save_item(db: State<'_, Arc<Database>>, item_id: String) -> Result<bool, String> {
    db.toggle_item_saved(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_item_seen(db: State<'_, Arc<Database>>, item_id: String) -> Result<(), String> {
    db.mark_item_seen(&item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn consume_pending_items(
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
pub async fn prune_old_items(db: State<'_, Arc<Database>>) -> Result<(i64, i64), String> {
    db.prune_old_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pending_count(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    db.get_pending_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_last_active_timestamp(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    let timestamp = db.get_last_active_timestamp().map_err(|e| e.to_string())?;
    Ok(timestamp.timestamp_millis())
}

#[tauri::command]
pub async fn set_last_active_timestamp(
    db: State<'_, Arc<Database>>,
    timestamp: i64,
) -> Result<(), String> {
    let datetime = chrono::DateTime::from_timestamp_millis(timestamp).ok_or("Invalid timestamp")?;
    db.set_last_active_timestamp(datetime)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_diagnostic_summary(db: State<'_, Arc<Database>>) -> Result<DiagnosticSummary, String> {
    db.get_diagnostic_summary().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_provider_status(db: State<'_, Arc<Database>>) -> Result<Vec<ProviderStatus>, String> {
    db.get_provider_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_diagnostics(
    db: State<'_, Arc<Database>>,
    limit: i64,
) -> Result<Vec<DiagnosticLog>, String> {
    db.get_recent_diagnostics(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_diagnostics(
    db: State<'_, Arc<Database>>,
    older_than_days: i64,
) -> Result<ClearDiagnosticsResult, String> {
    let deleted_count = db
        .clear_diagnostics(older_than_days)
        .map_err(|e| e.to_string())?;
    Ok(ClearDiagnosticsResult { deleted_count })
}

#[tauri::command]
pub async fn trigger_crawl(db: State<'_, Arc<Database>>) -> Result<u32, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    
    let providers: Vec<Box<dyn ContentProvider>> = vec![
        Box::new(crawler::reddit::RedditProvider::memes()),
        Box::new(crawler::reddit::RedditProvider::dad_jokes()),
        Box::new(crawler::reddit::RedditProvider::celebrity_gossip()),
        Box::new(crawler::dadjoke::DadJokeProvider),
        Box::new(crawler::meme::RedditMemeProvider),
        Box::new(crawler::video::RedditVideoProvider),
        Box::new(crawler::gossip::GossipProvider),
        Box::new(crawler::news::GoogleNewsRssProvider),
        Box::new(crawler::jokeapi::JokeApiProvider),
        Box::new(crawler::uselessfacts::UselessFactsProvider),
        Box::new(crawler::chucknorris::ChuckNorrisProvider),
        Box::new(crawler::hackernews::HackerNewsProvider),
        Box::new(crawler::bbcnews::BbcNewsProvider),
    ];
    
    let count = crawler::providers_per_cycle();
    let provider_idx = rand::random::<usize>() % providers.len();
    let mut items_added = 0u32;
    
    for i in 0..count {
        let idx = (provider_idx + i) % providers.len();
        let provider = &providers[idx];
        let provider_name = provider.name();

        let items = provider.fetch(&client).await;
        let item_count = items.len();

        for item in items {
            let crawl_item = item.into_crawl_item();
            match db.insert_item(&crawl_item) {
                Ok(true) => items_added += 1,
                Ok(false) => {}
                Err(e) => {
                    let _ = db.log_diagnostic_event(
                        "insert_error",
                        "warn",
                        &format!("Failed to insert item: {}", e),
                        None,
                        None
                    );
                }
            }
        }

        if item_count > 0 {
            let _ = db.log_diagnostic_event(
                "crawl_success",
                "info",
                &format!("{}: fetched {} items", provider_name, item_count),
                None,
                None
            );
        } else {
            let _ = db.log_diagnostic_event(
                "crawl_error",
                "warn",
                &format!("{}: no items fetched", provider_name),
                None,
                None
            );
        }
    }

    let _ = db.log_diagnostic_event(
        "crawl_complete",
        "info",
        &format!("Triggered crawl complete: {} new items", items_added),
        None,
        None
    );
    Ok(items_added)
}

#[tauri::command]
pub async fn log_diagnostic(
    db: State<'_, Arc<Database>>,
    event_type: String,
    severity: String,
    message: String,
    metadata: Option<String>,
) -> Result<(), String> {
    db.log_diagnostic_event(&event_type, &severity, &message, metadata.as_deref(), None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_image(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://www.reddit.com/")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch image: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Image fetch failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    let mime = if url.contains(".png") {
        "image/png"
    } else if url.contains(".gif") {
        "image/gif"
    } else if url.contains(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    let base64 = STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, base64))
}
