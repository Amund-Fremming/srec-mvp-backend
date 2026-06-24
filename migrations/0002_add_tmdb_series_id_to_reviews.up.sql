ALTER TABLE reviews ADD COLUMN tmdb_series_id BIGINT;
CREATE INDEX idx_reviews_user_tmdb ON reviews (user_id, tmdb_series_id);
