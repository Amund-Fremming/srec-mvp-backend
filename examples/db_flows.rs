use backend::adapters::db::adapter::DbAdapter;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let db = DbAdapter::new(pool);

    let series_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // --- Reviews ---
    // First save a recommendation so we can verify was_recommended flag
    let tmdb_series_id: i64 = 12345;
    let _ = db.save_recommendation(tmdb_series_id, 90).await;

    let was_recommended = db
        .is_series_recommended(tmdb_series_id)
        .await
        .unwrap_or(false);

    match db
        .save_review(
            series_id,
            user_id,
            tmdb_series_id,
            8,
            Some("Great pacing".into()),
            None,
            was_recommended,
        )
        .await
    {
        Ok(r) => println!(
            "✅ Saved review (id={}, was_recommended={})",
            r.id, r.was_recommended
        ),
        Err(e) => println!("❌ Failed to save review: {e}"),
    }

    match db.get_reviews(series_id).await {
        Ok(reviews) => {
            println!("✅ Fetched {} review(s)", reviews.len());
            for r in &reviews {
                match db.delete_review(r.id).await {
                    Ok(_) => println!("✅ Deleted review {}", r.id),
                    Err(e) => println!("❌ Failed to delete review {}: {e}", r.id),
                }
            }
        }
        Err(e) => println!("❌ Failed to fetch reviews: {e}"),
    }

    match db.get_reviews(series_id).await {
        Ok(reviews) => println!("✅ Reviews after delete: {}", reviews.len()),
        Err(e) => println!("❌ Failed to fetch reviews after delete: {e}"),
    }

    // --- Recommendations ---
    match db.get_recommendations().await {
        Ok(recs) => {
            println!("✅ Fetched {} recommendation(s)", recs.len());
            for r in &recs {
                match db.delete_recommendation(r.id).await {
                    Ok(_) => println!("✅ Deleted recommendation {}", r.id),
                    Err(e) => println!("❌ Failed to delete recommendation {}: {e}", r.id),
                }
            }
        }
        Err(e) => println!("❌ Failed to fetch recommendations: {e}"),
    }

    match db.get_recommendations().await {
        Ok(recs) => println!("✅ Recommendations after delete: {}", recs.len()),
        Err(e) => println!("❌ Failed to fetch recommendations after delete: {e}"),
    }

    // --- Users ---
    match db.create_user("bob", "pass123").await {
        Ok(id) => println!("\n✅ create_user → user id: {id}"),
        Err(e) => println!("\n❌ create_user failed: {e}"),
    }

    // Duplicate username should fail
    match db.create_user("bob", "pass123").await {
        Err(e) => println!("✅ Correctly rejected duplicate username: {e}"),
        Ok(_) => println!("❌ Duplicate username was accepted"),
    }
}
