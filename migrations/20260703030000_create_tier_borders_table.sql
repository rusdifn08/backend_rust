CREATE TABLE IF NOT EXISTS tier_borders (
    tier INT PRIMARY KEY,
    image_data BYTEA NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
