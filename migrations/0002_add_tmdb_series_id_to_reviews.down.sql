DROP INDEX IF EXISTS idx_reviews_user_tmdb;
ALTER TABLE reviews DROP COLUMN tmdb_series_id;
