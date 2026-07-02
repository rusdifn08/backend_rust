ALTER TABLE users
ADD COLUMN IF NOT EXISTS friend_code VARCHAR(6) UNIQUE,
ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(255);

-- Generate unique 6-character hex strings for existing users
UPDATE users 
SET friend_code = upper(substring(md5(random()::text), 1, 6))
WHERE friend_code IS NULL;

-- Make friend_code NOT NULL after backfilling
ALTER TABLE users
ALTER COLUMN friend_code SET NOT NULL;
