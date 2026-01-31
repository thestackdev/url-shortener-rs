CREATE TABLE IF NOT EXISTS urls (
  short_code TEXT PRIMARY KEY,
  original_url TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER,
  visits INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON urls(expires_at);
