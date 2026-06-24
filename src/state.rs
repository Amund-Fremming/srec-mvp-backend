use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::adapters::db::DbAdapter;
use crate::adapters::llm::adapter::LlmAdapter;
use crate::adapters::tmdb::adapter::TmdbAdapter;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub db: DbAdapter,
    pub llm: LlmAdapter,
    pub tmdb: TmdbAdapter,
}

impl AppState {
    pub async fn new(
        database_url: &str,
        openai_api_key: String,
        tmdb_access_token: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let db = DbAdapter::new(pool.clone());
        let llm = LlmAdapter::new(openai_api_key);
        let tmdb = TmdbAdapter::new(tmdb_access_token);

        Ok(Self {
            pool,
            db,
            llm,
            tmdb,
        })
    }
}
