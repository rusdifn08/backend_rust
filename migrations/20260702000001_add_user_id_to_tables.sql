ALTER TABLE notes ADD COLUMN user_id UUID;
ALTER TABLE todos ADD COLUMN user_id UUID;
ALTER TABLE habits ADD COLUMN user_id UUID;
ALTER TABLE focus_sessions ADD COLUMN user_id UUID;
ALTER TABLE transactions ADD COLUMN user_id UUID;
