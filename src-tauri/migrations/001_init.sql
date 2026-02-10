CREATE TABLE IF NOT EXISTS crawl_items (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    thumbnail_url TEXT,
    description TEXT,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_seen INTEGER NOT NULL DEFAULT 0,
    is_saved INTEGER NOT NULL DEFAULT 0,
    session_date TEXT NOT NULL DEFAULT (date('now'))
);

CREATE INDEX IF NOT EXISTS idx_crawl_items_session ON crawl_items(session_date);
CREATE INDEX IF NOT EXISTS idx_crawl_items_category ON crawl_items(category);
CREATE INDEX IF NOT EXISTS idx_crawl_items_fetched ON crawl_items(fetched_at);

CREATE TABLE IF NOT EXISTS session_stats (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL UNIQUE,
    started_at TEXT NOT NULL,
    memes_found INTEGER NOT NULL DEFAULT 0,
    jokes_found INTEGER NOT NULL DEFAULT 0,
    news_checked INTEGER NOT NULL DEFAULT 0,
    videos_found INTEGER NOT NULL DEFAULT 0,
    gossip_found INTEGER NOT NULL DEFAULT 0,
    total_items INTEGER NOT NULL DEFAULT 0,
    estimated_time_saved_minutes REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE IF NOT EXISTS notification_log (
    id TEXT PRIMARY KEY,
    message TEXT NOT NULL,
    sent_at TEXT NOT NULL DEFAULT (datetime('now')),
    related_item_id TEXT,
    FOREIGN KEY (related_item_id) REFERENCES crawl_items(id)
);
