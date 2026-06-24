use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub series_id: Uuid,
    pub user_id: Uuid,
    pub tmdb_series_id: Option<i64>,
    pub rating: i16,
    pub liked: Option<String>,
    pub disliked: Option<String>,
    pub was_recommended: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub passcode: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Recommendation {
    pub id: Uuid,
    pub tmdb_series_id: i64,
    pub confidence: i16,
    pub created_at: DateTime<Utc>,
}
