-- Migration 003: Add app_state table for tracking app lifecycle
-- This table stores key-value pairs for app state management

CREATE TABLE IF NOT EXISTS app_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial last_active_timestamp if not exists
INSERT OR IGNORE INTO app_state (key, value) VALUES ('last_active_timestamp', strftime('%s', 'now') * 1000);
