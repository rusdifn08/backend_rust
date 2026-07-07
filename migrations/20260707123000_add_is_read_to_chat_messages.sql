ALTER TABLE chat_messages
ADD COLUMN IF NOT EXISTS is_read BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_chat_messages_participants_created_at
ON chat_messages (sender_id, receiver_id, created_at);
