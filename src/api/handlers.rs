use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    adapters::tmdb::models::{SeriesDetails, SeriesListItem},
    errors::AppError,
    models::{
        CreateUserRequest, LoginRequest, PagedQuery, ReviewDto, ReviewRequest, Series,
        UpdateReviewRequest,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .route("/series", get(get_series_page))
        .route("/series/search", get(search_series))
        .route("/series/review", get(get_user_review).post(save_review))
        .route(
            "/series/review/{review_id}",
            delete(delete_review).patch(patch_review),
        )
        .route("/series/reviews/{user_id}", get(get_user_reviews))
        .route("/series/{tmdb_id}", get(get_series_by_tmdb_id))
        .route(
            "/series/recommendations/{user_id}",
            get(get_stored_recommendations).post(get_llm_recommendations),
        )
        .route("/users", post(create_user))
}

#[derive(Deserialize, ToSchema)]
pub struct SearchParams {
    pub q: String,
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "API is healthy")), tag = "health")]
pub async fn health(State(_state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(StatusCode::OK)
}

#[utoipa::path(post, path = "/login", request_body = LoginRequest, responses((status = 200, description = "User ID")), tag = "auth")]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = state
        .db
        .login(&body.username.to_lowercase(), &body.passcode)
        .await?;

    Ok((StatusCode::OK, Json(user_id)))
}

#[utoipa::path(get, path = "/series", responses((status = 200, description = "List of series", body = Vec<Series>)), tag = "series")]
pub async fn get_series_page(
    State(state): State<AppState>,
    Query(q): Query<PagedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = state.tmdb.get_popular_series(q.page()).await?;

    Ok((StatusCode::OK, Json(page)))
}

#[utoipa::path(get, path = "/series/search", params(("q" = String, Query, description = "Search query")), responses((status = 200, description = "Matching series", body = Vec<SeriesListItem>)), tag = "series")]
pub async fn search_series(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    let results = state.tmdb.search_series(&params.q).await?;
    Ok((StatusCode::OK, Json(results)))
}

#[utoipa::path(post, path = "/series/review", request_body = ReviewRequest, responses((status = 201, description = "Review saved", body = ReviewDto)), tag = "series")]
pub async fn save_review(
    State(state): State<AppState>,
    Json(payload): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::ValidationError(errors.to_string()));
    }

    let was_recommended = state
        .db
        .is_series_recommended(payload.tmdb_series_id)
        .await?;

    let review = state
        .db
        .save_review(
            payload.series_id,
            payload.user_id,
            payload.tmdb_series_id,
            payload.rating,
            payload.liked,
            payload.disliked,
            was_recommended,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(ReviewDto::from(review))))
}

#[derive(Deserialize, ToSchema)]
pub struct UserReviewParams {
    pub user_id: Uuid,
    pub tmdb_series_id: i64,
}

#[utoipa::path(get, path = "/series/review", params(("user_id" = Uuid, Query, description = "User UUID"), ("tmdb_series_id" = i64, Query, description = "TMDB series ID")), responses((status = 200, description = "User review or null", body = Option<ReviewDto>)), tag = "series")]
pub async fn get_user_review(
    State(state): State<AppState>,
    Query(params): Query<UserReviewParams>,
) -> Result<impl IntoResponse, AppError> {
    let review = state
        .db
        .get_user_review(params.user_id, params.tmdb_series_id)
        .await?;

    Ok((StatusCode::OK, Json(review.map(ReviewDto::from))))
}

#[utoipa::path(delete, path = "/series/review/{review_id}", params(("review_id" = Uuid, Path, description = "Review UUID")), responses((status = 204, description = "Review deleted")), tag = "series")]
pub async fn delete_review(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.db.delete_review(review_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/series/review/{review_id}", request_body = UpdateReviewRequest, params(("review_id" = Uuid, Path, description = "Review UUID")), responses((status = 200, description = "Updated review", body = ReviewDto), (status = 404, description = "Review not found")), tag = "series")]
pub async fn patch_review(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
    Json(payload): Json<UpdateReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(errors) = payload.validate() {
        return Err(AppError::ValidationError(errors.to_string()));
    }

    let review = state
        .db
        .update_review(review_id, payload.rating, payload.liked, payload.disliked)
        .await?;

    Ok((StatusCode::OK, Json(ReviewDto::from(review))))
}

#[utoipa::path(get, path = "/series/reviews/{user_id}", params(("user_id" = Uuid, Path, description = "User UUID")), responses((status = 200, description = "All reviews for user", body = Vec<ReviewDto>)), tag = "series")]
pub async fn get_user_reviews(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let reviews = state.db.get_user_reviews(user_id).await?;
    let dtos: Vec<ReviewDto> = reviews.into_iter().map(ReviewDto::from).collect();
    Ok((StatusCode::OK, Json(dtos)))
}

#[utoipa::path(get, path = "/series/{tmdb_id}", params(("tmdb_id" = u64, Path, description = "TMDB series ID")), responses((status = 200, description = "Series details from TMDB")), tag = "series")]
pub async fn get_series_by_tmdb_id(
    State(state): State<AppState>,
    Path(tmdb_id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let details = state.tmdb.get_series_details(tmdb_id).await?;
    Ok((StatusCode::OK, Json(details)))
}

/// A previously stored recommendation enriched with its TMDB series details and
/// the confidence score the LLM assigned when it generated the match.
#[derive(Serialize)]
pub struct StoredRecommendation {
    #[serde(flatten)]
    pub series: SeriesDetails,
    pub confidence: i16,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(get, path = "/series/recommendations/{user_id}", params(("user_id" = Uuid, Path, description = "User UUID")), responses((status = 200, description = "Stored recommendations as TMDB series with confidence, newest first")), tag = "series")]
pub async fn get_stored_recommendations(
    State(state): State<AppState>,
    Path(_user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Already ordered by created_at DESC (newest first) in the query.
    let recs = state.db.get_recommendations().await?;

    let mut stored = Vec::new();
    for rec in recs {
        if let Ok(series) = state
            .tmdb
            .get_series_details(rec.tmdb_series_id as u64)
            .await
        {
            stored.push(StoredRecommendation {
                series,
                confidence: rec.confidence,
                created_at: rec.created_at,
            });
        }
    }

    Ok(Json(stored))
}

#[utoipa::path(post, path = "/series/recommendations/{user_id}", params(("user_id" = Uuid, Path, description = "User UUID")), responses((status = 200, description = "Generated recommendations as TMDB series")), tag = "series")]
pub async fn get_llm_recommendations(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let reviews = state.db.get_user_reviews(user_id).await?;

    if reviews.is_empty() {
        return Ok((StatusCode::OK, Json(Vec::<SeriesListItem>::new())));
    }

    let mut reviews_with_titles = Vec::new();
    for review in reviews {
        if let Some(tmdb_id) = review.tmdb_series_id {
            if let Ok(details) = state.tmdb.get_series_details(tmdb_id as u64).await {
                reviews_with_titles.push((review, details.name));
            }
        }
    }

    if reviews_with_titles.is_empty() {
        return Ok((StatusCode::OK, Json(Vec::<SeriesListItem>::new())));
    }

    let llm_recs = state.llm.recommend(&reviews_with_titles).await?;

    let mut series_results = Vec::new();
    for rec in llm_recs.recommendations {
        if let Ok(results) = state.tmdb.search_series(&rec.title).await {
            if let Some(series) = results.into_iter().next() {
                let _ = state
                    .db
                    .save_recommendation(series.id as i64, rec.confidence as i16)
                    .await;
                series_results.push(series);
            }
        }
    }

    Ok((StatusCode::OK, Json(series_results)))
}

#[utoipa::path(post, path = "/users", request_body = CreateUserRequest, responses((status = 201, description = "User created"), (status = 409, description = "Username already exists")), tag = "users")]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let id = state
        .db
        .create_user(&body.username.to_lowercase(), &body.passcode)
        .await?;
    Ok((StatusCode::CREATED, Json(id)))
}
