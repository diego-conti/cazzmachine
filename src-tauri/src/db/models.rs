use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlItem {
    pub id: String,
    pub source: String,
    pub category: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub fetched_at: String,
    pub is_seen: bool,
    pub is_saved: bool,
    pub is_consumed: bool,
    pub session_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayStats {
    pub memes_found: i64,
    pub jokes_found: i64,
    pub news_checked: i64,
    pub videos_found: i64,
    pub gossip_found: i64,
    pub total_items: i64,
    pub estimated_time_saved_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumeResult {
    pub items_consumed: i64,
    pub items_discarded: i64,
    pub time_consumed_minutes: f64,
    pub memes_consumed: i64,
    pub jokes_consumed: i64,
    pub news_consumed: i64,
    pub videos_consumed: i64,
    pub gossip_consumed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySummary {
    pub stats: DayStats,
    pub summary_text: String,
    pub highlights: Vec<CrawlItem>,
}
