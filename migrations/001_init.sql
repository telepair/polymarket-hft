-- Initialize database schema for Polymarket HFT
-- This script runs on first container startup

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create metrics table
CREATE TABLE IF NOT EXISTS metrics (
    time TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    name TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB DEFAULT '{}'
);

-- Convert to hypertable
SELECT create_hypertable('metrics', 'time', if_not_exists => TRUE);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_metrics_source_name_time 
    ON metrics (source, name, time DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_labels 
    ON metrics USING GIN (labels);

-- Create continuous aggregate for hourly rollup (optional, for dashboards)
-- Uncomment if needed:
-- CREATE MATERIALIZED VIEW metrics_hourly
-- WITH (timescaledb.continuous) AS
-- SELECT
--     time_bucket('1 hour', time) AS bucket,
--     source,
--     name,
--     avg(value) AS avg_value,
--     min(value) AS min_value,
--     max(value) AS max_value,
--     count(*) AS sample_count
-- FROM metrics
-- GROUP BY bucket, source, name;

-- Enable compression for data older than 7 days (optional)
-- ALTER TABLE metrics SET (
--     timescaledb.compress,
--     timescaledb.compress_segmentby = 'source,name'
-- );
-- SELECT add_compression_policy('metrics', INTERVAL '7 days');

--------------------------------------------------------------------------------
-- scrape_jobs table for storing scrape task configurations
-- This table stores configurations that can be managed via the web UI
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS scrape_jobs (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    params JSONB DEFAULT '{}',
    targets TEXT[] NOT NULL,
    schedule_type TEXT NOT NULL CHECK (schedule_type IN ('interval', 'cron')),
    schedule_value TEXT NOT NULL,
    state_ttl_secs BIGINT,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_source ON scrape_jobs (source);
CREATE INDEX IF NOT EXISTS idx_scrape_jobs_enabled ON scrape_jobs (enabled);

-- Add a trigger to update updated_at on modification
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_scrape_jobs_updated_at ON scrape_jobs;
CREATE TRIGGER update_scrape_jobs_updated_at
    BEFORE UPDATE ON scrape_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
