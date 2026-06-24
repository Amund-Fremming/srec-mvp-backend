use backend::adapters::tmdb::adapter::TmdbAdapter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let access_token =
        std::env::var("TMDB_ACCESS_TOKEN").expect("TMDB_ACCESS_TOKEN must be set in .env");

    let tmdb = TmdbAdapter::new(access_token);

    // --- Search ---
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Breaking Bad".to_string());

    let results = match tmdb.search_series(&query).await {
        Ok(r) if r.is_empty() => {
            println!("❌ No results found for: {query}");
            return;
        }
        Ok(r) => {
            println!("✅ Found {} result(s) for: {query}", r.len());
            r
        }
        Err(e) => {
            println!("❌ Search failed: {e}");
            return;
        }
    };

    for (i, s) in results.iter().take(5).enumerate() {
        println!(
            "  {}. {} ({}) — rating: {:.1}/10 ({} votes)",
            i + 1,
            s.name,
            s.first_air_date.as_deref().unwrap_or("unknown"),
            s.vote_average,
            s.vote_count,
        );
        if !s.overview.is_empty() {
            let preview: String = s.overview.chars().take(120).collect();
            println!("     {preview}...");
        }
    }

    // --- Full details for the top result ---
    let top = &results[0];

    let details = match tmdb.get_series_details(top.id).await {
        Ok(d) => {
            println!("\n✅ Fetched details for: {} (id={})", top.name, top.id);
            d
        }
        Err(e) => {
            println!("❌ Failed to fetch details for {}: {e}", top.name);
            return;
        }
    };

    let genres: Vec<&str> = details.genres.iter().map(|g| g.name.as_str()).collect();
    let networks: Vec<&str> = details.networks.iter().map(|n| n.name.as_str()).collect();

    println!("  Status:   {}", details.status);
    println!(
        "  Aired:    {} → {}",
        details.first_air_date.as_deref().unwrap_or("?"),
        details.last_air_date.as_deref().unwrap_or("ongoing")
    );
    println!(
        "  Seasons:  {} ({} episodes total)",
        details.number_of_seasons, details.number_of_episodes
    );
    println!(
        "  Rating:   {:.1}/10 ({} votes)",
        details.vote_average, details.vote_count
    );
    println!("  Genres:   {}", genres.join(", "));
    println!("  Networks: {}", networks.join(", "));
    println!("  Overview: {}", details.overview);

    // --- Popular series ---
    match tmdb.get_popular_series(1).await {
        Ok(popular) => {
            println!("\n✅ Popular series (page 1): {} result(s)", popular.len());
            for (i, s) in popular.iter().take(3).enumerate() {
                println!("  {}. {} — rating: {:.1}/10", i + 1, s.name, s.vote_average);
            }
        }
        Err(e) => println!("\n❌ Failed to fetch popular series: {e}"),
    }

    // --- Watch providers for Norway ---
    match tmdb.get_watch_providers(top.id, "NO").await {
        Ok(Some(providers)) => match &providers.streaming {
            Some(streaming) => {
                let names: Vec<&str> = streaming.iter().map(|p| p.provider_name.as_str()).collect();
                println!("\n✅ Streaming in Norway: {}", names.join(", "));
            }
            None => println!("\n❌ Not available for streaming in Norway."),
        },
        Ok(None) => println!("\n❌ No watch provider data available for Norway."),
        Err(e) => println!("\n❌ Failed to fetch watch providers: {e}"),
    }
}
