use sqlx::{Pool, Postgres};
use thiserror::Error;
use uuid::Uuid;

use super::models::{Recommendation, Review, User};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Username already exists: {0}")]
    UsernameConflict(String),

    #[error("Not found")]
    NotFound,
}

#[derive(Clone)]
pub struct DbAdapter {
    pool: Pool<Postgres>,
}

impl DbAdapter {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn is_series_recommended(&self, tmdb_series_id: i64) -> Result<bool, DbError> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM recommendations WHERE tmdb_series_id = $1) as "exists!""#,
            tmdb_series_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    pub async fn save_review(
        &self,
        series_id: Uuid,
        user_id: Uuid,
        tmdb_series_id: i64,
        rating: i16,
        liked: Option<String>,
        disliked: Option<String>,
        was_recommended: bool,
    ) -> Result<Review, DbError> {
        let review = sqlx::query_as!(
            Review,
            r#"
            INSERT INTO reviews (series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended as "was_recommended!", created_at
            "#,
            series_id,
            user_id,
            tmdb_series_id,
            rating,
            liked,
            disliked,
            was_recommended,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(review)
    }

    pub async fn get_reviews(&self, series_id: Uuid) -> Result<Vec<Review>, DbError> {
        let reviews = sqlx::query_as!(
            Review,
            r#"
            SELECT id, series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended as "was_recommended!", created_at
            FROM reviews
            WHERE series_id = $1
            ORDER BY created_at DESC
            "#,
            series_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reviews)
    }

    pub async fn get_user_reviews(&self, user_id: Uuid) -> Result<Vec<Review>, DbError> {
        let reviews = sqlx::query_as!(
            Review,
            r#"
            SELECT id, series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended as "was_recommended!", created_at
            FROM reviews
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reviews)
    }

    pub async fn get_user_review(
        &self,
        user_id: Uuid,
        tmdb_series_id: i64,
    ) -> Result<Option<Review>, DbError> {
        let review = sqlx::query_as!(
            Review,
            r#"
            SELECT id, series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended as "was_recommended!", created_at
            FROM reviews
            WHERE user_id = $1 AND tmdb_series_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id,
            tmdb_series_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(review)
    }

    pub async fn save_recommendation(
        &self,
        tmdb_series_id: i64,
        confidence: i16,
    ) -> Result<(), DbError> {
        // Re-recommending a series should surface it as new: drop any earlier row
        // for the same series and insert a fresh one so created_at and confidence
        // reflect the latest generation. Done in a transaction to stay atomic.
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM recommendations WHERE tmdb_series_id = $1"#,
            tmdb_series_id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO recommendations (tmdb_series_id, confidence)
            VALUES ($1, $2)
            "#,
            tmdb_series_id,
            confidence,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_recommendations(&self) -> Result<Vec<Recommendation>, DbError> {
        let recs = sqlx::query_as!(
            Recommendation,
            r#"
            SELECT id, tmdb_series_id, confidence, created_at
            FROM recommendations
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(recs)
    }

    pub async fn update_review(
        &self,
        id: Uuid,
        rating: i16,
        liked: Option<String>,
        disliked: Option<String>,
    ) -> Result<Review, DbError> {
        let review = sqlx::query_as!(
            Review,
            r#"
            UPDATE reviews
            SET rating = $2, liked = $3, disliked = $4
            WHERE id = $1
            RETURNING id, series_id, user_id, tmdb_series_id, rating, liked, disliked, was_recommended as "was_recommended!", created_at
            "#,
            id,
            rating,
            liked,
            disliked,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DbError::NotFound)?;

        Ok(review)
    }

    pub async fn delete_review(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            DELETE FROM reviews
            WHERE id = $1
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_recommendation(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            DELETE FROM recommendations
            WHERE id = $1
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_user(&self, username: &str, passcode: &str) -> Result<Uuid, DbError> {
        let existing =
            sqlx::query_scalar!(r#"SELECT id FROM users WHERE username = $1"#, username,)
                .fetch_optional(&self.pool)
                .await?;

        if existing.is_some() {
            return Err(DbError::UsernameConflict(username.to_string()));
        }

        let id = sqlx::query_scalar!(
            r#"INSERT INTO users (username, passcode) VALUES ($1, $2) RETURNING id"#,
            username,
            passcode,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn login_or_create_user(
        &self,
        username: &str,
        passcode: &str,
    ) -> Result<Option<Uuid>, DbError> {
        let existing = sqlx::query_as!(
            User,
            r#"SELECT id, username, passcode, created_at FROM users WHERE username = $1"#,
            username,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(user) = existing {
            if user.passcode != passcode {
                return Ok(None);
            }
            return Ok(Some(user.id));
        }

        let id = sqlx::query_scalar!(
            r#"INSERT INTO users (username, passcode) VALUES ($1, $2) RETURNING id"#,
            username,
            passcode,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(id))
    }
}
