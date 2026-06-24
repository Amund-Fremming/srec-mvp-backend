use backend::adapters::llm::models::SeriesRecommendations;
use backend::api::handlers;
use backend::models::{ReviewDto, ReviewRequest, Series};
use backend::state::AppState;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::login,
        handlers::get_series_page,
        handlers::search_series,
        handlers::save_review,
        handlers::get_user_review,
        handlers::delete_review,
        handlers::get_stored_recommendations,
        handlers::get_llm_recommendations,
    ),
    components(schemas(Series, ReviewRequest, ReviewDto, SeriesRecommendations)),
    tags(
        (name = "health", description = "Health check"),
        (name = "series", description = "Series endpoints"),
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let tmdb_access_token =
        std::env::var("TMDB_ACCESS_TOKEN").expect("TMDB_ACCESS_TOKEN must be set");

    let state = AppState::new(&database_url, openai_api_key, tmdb_access_token)
        .await
        .expect("failed to initialise app state");

    let app = handlers::router()
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
