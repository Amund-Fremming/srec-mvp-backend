use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::adapters::db::models::Review;

const DEFAULT_PAGE: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Series {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub genre: String,
    pub year: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ReviewRequest {
    pub series_id: Uuid,
    pub user_id: Uuid,
    pub tmdb_series_id: i64,
    #[validate(range(min = 1, max = 10))]
    pub rating: i16,
    pub liked: Option<String>,
    pub disliked: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReviewDto {
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

impl From<Review> for ReviewDto {
    fn from(r: Review) -> Self {
        Self {
            id: r.id,
            series_id: r.series_id,
            user_id: r.user_id,
            tmdb_series_id: r.tmdb_series_id,
            rating: r.rating,
            liked: r.liked,
            disliked: r.disliked,
            was_recommended: r.was_recommended,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateReviewRequest {
    #[validate(range(min = 1, max = 10))]
    pub rating: i16,
    pub liked: Option<String>,
    pub disliked: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub passcode: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub passcode: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PagedQuery {
    page: Option<u8>,
}

impl PagedQuery {
    pub fn page(&self) -> u8 {
        let p = self.page.unwrap_or(DEFAULT_PAGE);
        if p == 0 { 1 } else { p }
    }
}
