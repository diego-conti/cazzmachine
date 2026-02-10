ALTER TABLE crawl_items ADD COLUMN is_consumed INTEGER NOT NULL DEFAULT 0;
UPDATE crawl_items SET is_consumed = 1;
CREATE INDEX IF NOT EXISTS idx_crawl_items_consumed ON crawl_items(is_consumed);
