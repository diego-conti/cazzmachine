pub mod models;

use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> SqlResult<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("cazzmachine.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_init.sql"))?;

        let has_consumed: bool = conn
            .prepare(
                "SELECT COUNT(*) FROM pragma_table_info('crawl_items') WHERE name='is_consumed'",
            )?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_consumed {
            conn.execute_batch(include_str!("../../migrations/002_add_is_consumed.sql"))?;
        }

        conn.execute(
            "UPDATE crawl_items SET is_consumed = 1 WHERE is_consumed = 0 AND fetched_at < datetime('now', '-1 minute')",
            [],
        )?;

        // Run migration 003 for app_state table
        conn.execute_batch(include_str!("../../migrations/003_add_app_state.sql"))?;

        Ok(())
    }

    pub fn insert_item(&self, item: &models::CrawlItem) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            "INSERT OR IGNORE INTO crawl_items (id, source, category, title, url, thumbnail_url, description, fetched_at, session_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                item.id,
                item.source,
                item.category,
                item.title,
                item.url,
                item.thumbnail_url,
                item.description,
                item.fetched_at,
                item.session_date,
            ],
        );
        match result {
            Ok(rows) => Ok(rows > 0),
            Err(e) => Err(e),
        }
    }

    fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<models::CrawlItem> {
        Ok(models::CrawlItem {
            id: row.get(0)?,
            source: row.get(1)?,
            category: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            thumbnail_url: row.get(5)?,
            description: row.get(6)?,
            fetched_at: row.get(7)?,
            is_seen: row.get(8)?,
            is_saved: row.get(9)?,
            is_consumed: row.get(10)?,
            session_date: row.get(11)?,
        })
    }

    const ITEM_COLUMNS: &'static str =
        "id, source, category, title, url, thumbnail_url, description, fetched_at, is_seen, is_saved, is_consumed, session_date";

    pub fn get_items_for_today(&self) -> SqlResult<Vec<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND is_consumed = 1 ORDER BY fetched_at DESC",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let items = stmt
            .query_map(rusqlite::params![today], Self::row_to_item)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_items_by_category(&self, category: &str) -> SqlResult<Vec<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND category = ?2 AND is_consumed = 1 ORDER BY fetched_at DESC",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let items = stmt
            .query_map(rusqlite::params![today, category], Self::row_to_item)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_today_stats(&self) -> SqlResult<models::DayStats> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) FROM crawl_items WHERE session_date = ?1 AND is_consumed = 1 GROUP BY category",
        )?;
        let mut stats = models::DayStats::default();
        let rows = stmt.query_map(rusqlite::params![today], |row| {
            let category: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((category, count))
        })?;
        for row in rows {
            let (category, count) = row?;
            match category.as_str() {
                "meme" => stats.memes_found = count,
                "joke" => stats.jokes_found = count,
                "news" => stats.news_checked = count,
                "video" => stats.videos_found = count,
                "gossip" => stats.gossip_found = count,
                _ => {}
            }
            stats.total_items += count;
        }
        stats.estimated_time_saved_minutes = Self::estimate_time_saved(&stats);
        Ok(stats)
    }

    pub fn get_latest_unseen_item(&self) -> SqlResult<Option<models::CrawlItem>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {} FROM crawl_items WHERE session_date = ?1 AND is_seen = 0 AND is_consumed = 1 ORDER BY fetched_at DESC LIMIT 1",
            Self::ITEM_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut items = stmt.query_map(rusqlite::params![today], Self::row_to_item)?;
        match items.next() {
            Some(Ok(item)) => Ok(Some(item)),
            _ => Ok(None),
        }
    }

    pub fn consume_pending_items(&self, budget_minutes: f64) -> SqlResult<models::ConsumeResult> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let mut stmt = conn.prepare(
            "SELECT id, category FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0 ORDER BY fetched_at ASC",
        )?;
        let pending: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![today], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        let mut remaining_budget = budget_minutes;
        let mut consumed_ids: Vec<String> = Vec::new();
        let mut time_consumed = 0.0_f64;
        let mut memes: i64 = 0;
        let mut jokes: i64 = 0;
        let mut news: i64 = 0;
        let mut videos: i64 = 0;
        let mut gossip: i64 = 0;

        for (id, category) in &pending {
            let cost = Self::category_cost(category);
            if remaining_budget >= cost {
                consumed_ids.push(id.clone());
                remaining_budget -= cost;
                time_consumed += cost;
                match category.as_str() {
                    "meme" => memes += 1,
                    "joke" => jokes += 1,
                    "news" => news += 1,
                    "video" => videos += 1,
                    "gossip" => gossip += 1,
                    _ => {}
                }
            }
            // Unconsumed items stay in buffer for next phase - no deletion
        }

        for id in &consumed_ids {
            conn.execute(
                "UPDATE crawl_items SET is_consumed = 1 WHERE id = ?1",
                rusqlite::params![id],
            )?;
        }

        let unconsumed_count = pending.len() as i64 - consumed_ids.len() as i64;

        Ok(models::ConsumeResult {
            items_consumed: consumed_ids.len() as i64,
            items_discarded: unconsumed_count,
            time_consumed_minutes: time_consumed,
            memes_consumed: memes,
            jokes_consumed: jokes,
            news_consumed: news,
            videos_consumed: videos,
            gossip_consumed: gossip,
        })
    }

    fn category_cost(category: &str) -> f64 {
        match category {
            "meme" => 0.5,
            "joke" => 0.3,
            "news" => 2.0,
            "video" => 3.0,
            "gossip" => 1.5,
            _ => 1.0,
        }
    }

    pub fn mark_item_seen(&self, item_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE crawl_items SET is_seen = 1 WHERE id = ?1",
            rusqlite::params![item_id],
        )?;
        Ok(())
    }

    pub fn toggle_item_saved(&self, item_id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE crawl_items SET is_saved = CASE WHEN is_saved = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            rusqlite::params![item_id],
        )?;
        let saved: bool = conn.query_row(
            "SELECT is_saved FROM crawl_items WHERE id = ?1",
            rusqlite::params![item_id],
            |row| row.get(0),
        )?;
        Ok(saved)
    }

    fn estimate_time_saved(stats: &models::DayStats) -> f64 {
        // Average time a human spends per content type (in minutes)
        let meme_time = 0.5; // 30 sec per meme
        let joke_time = 0.3; // 20 sec per joke
        let news_time = 2.0; // 2 min per news article
        let video_time = 3.0; // 3 min per video
        let gossip_time = 1.5; // 1.5 min per gossip item

        (stats.memes_found as f64 * meme_time)
            + (stats.jokes_found as f64 * joke_time)
            + (stats.news_checked as f64 * news_time)
            + (stats.videos_found as f64 * video_time)
            + (stats.gossip_found as f64 * gossip_time)
    }

    pub fn log_notification(&self, message: &str, related_item_id: Option<&str>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO notification_log (id, message, related_item_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, message, related_item_id],
        )?;
        Ok(())
    }

    pub fn get_pending_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM crawl_items WHERE session_date = ?1 AND is_consumed = 0",
            rusqlite::params![today],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn prune_old_items(&self) -> SqlResult<(i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let deleted = conn.execute(
            "DELETE FROM crawl_items WHERE session_date < ?1 AND is_consumed = 0",
            rusqlite::params![today],
        )?;

        let stripped = conn.execute(
            "UPDATE crawl_items SET title = '[ARCHIVED]', description = NULL, thumbnail_url = NULL, is_seen = 1 WHERE session_date < ?1 AND is_consumed = 1",
            rusqlite::params![today],
        )?;

        Ok((deleted as i64, stripped as i64))
    }

    pub fn get_last_active_timestamp(&self) -> SqlResult<chrono::DateTime<chrono::Utc>> {
        let conn = self.conn.lock().unwrap();

        // Try to get from app_state table first
        let result: Option<i64> = conn
            .query_row(
                "SELECT value FROM app_state WHERE key = 'last_active_timestamp'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(timestamp) = result {
            Ok(chrono::DateTime::from_timestamp_millis(timestamp)
                .unwrap_or_else(|| chrono::Utc::now()))
        } else {
            // Return current time if not set
            Ok(chrono::Utc::now())
        }
    }

    pub fn set_last_active_timestamp(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp_millis = timestamp.timestamp_millis();

        conn.execute(
            "INSERT OR REPLACE INTO app_state (key, value) VALUES ('last_active_timestamp', ?1)",
            rusqlite::params![timestamp_millis],
        )?;

        Ok(())
    }
}
