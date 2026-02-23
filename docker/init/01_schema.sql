CREATE TABLE IF NOT EXISTS mentions (
  id BIGSERIAL PRIMARY KEY,
  guild_id BIGINT NOT NULL,
  channel_id BIGINT NOT NULL,
  message_id BIGINT NOT NULL UNIQUE,
  author_id BIGINT NOT NULL,
  content TEXT NOT NULL,
  mention_everyone BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mentions_author_created_at
  ON mentions (author_id, created_at DESC);

CREATE TABLE IF NOT EXISTS mention_targets (
  mention_id BIGINT NOT NULL REFERENCES mentions(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL,
  extended_until BIGINT NULL,
  ignored_at BIGINT NULL,
  PRIMARY KEY (mention_id, user_id)
);

ALTER TABLE mention_targets ADD COLUMN IF NOT EXISTS extended_until BIGINT NULL;
ALTER TABLE mention_targets ADD COLUMN IF NOT EXISTS ignored_at BIGINT NULL;

CREATE INDEX IF NOT EXISTS idx_mention_targets_user
  ON mention_targets (user_id);

CREATE TABLE IF NOT EXISTS mention_reads (
  mention_id BIGINT NOT NULL REFERENCES mentions(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL,
  read_at BIGINT NOT NULL,
  PRIMARY KEY (mention_id, user_id)
);

CREATE TABLE IF NOT EXISTS mention_dones (
  mention_id BIGINT NOT NULL REFERENCES mentions(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL,
  done_at BIGINT NOT NULL,
  PRIMARY KEY (mention_id, user_id)
);
