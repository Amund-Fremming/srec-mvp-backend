use backend::adapters::db::models::Review;
use backend::adapters::llm::adapter::LlmAdapter;
use chrono::Utc;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in .env");

    let adapter = LlmAdapter::new(api_key);

    let reviews = vec![
        (
            Review {
                id: Uuid::new_v4(),
                series_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                tmdb_series_id: Some(1396),
                rating: 9,
                liked: Some("Great writing, complex characters".into()),
                disliked: None,
                was_recommended: false,
                created_at: Utc::now(),
            },
            "Breaking Bad".to_string(),
        ),
        (
            Review {
                id: Uuid::new_v4(),
                series_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                tmdb_series_id: Some(69740),
                rating: 7,
                liked: Some("Interesting premise".into()),
                disliked: Some("Too slow in the middle".into()),
                was_recommended: false,
                created_at: Utc::now(),
            },
            "Ozark".to_string(),
        ),
    ];

    match adapter.recommend(&reviews).await {
        Ok(recs) => {
            println!("✅ Taste profile: {}\n", recs.taste_summary);
            for (i, r) in recs.recommendations.iter().enumerate() {
                println!(
                    "✅ {}. {} [{}] — confidence: {}%",
                    i + 1,
                    r.title,
                    r.genre,
                    r.confidence
                );
            }
        }
        Err(e) => println!("❌ Recommendation failed: {e}"),
    }
}
