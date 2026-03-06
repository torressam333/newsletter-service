-- Wrap the migration in a transaction to ensure it succeeds for fails automagically
-- NOTE: sqlx does not do transactions auto for us
BEGIN;
  -- Back fill status for historical entries
  UPDATE subscriptions
    SET status = 'confirmed'
    WHERE status IS NULL;
  -- Make status mandatory
  ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
