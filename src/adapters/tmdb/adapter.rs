use std::time::Duration;

use reqwest::Client;
use thiserror::Error;

use super::models::{CountryProviders, SeriesDetails, SeriesListItem, WatchProvidersResponse};

const BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Error)]
pub enum TmdbError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned an error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct TmdbAdapter {
    client: Client,
    access_token: String,
}

impl TmdbAdapter {
    pub fn new(access_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            access_token,
        }
    }

    pub async fn search_series(&self, query: &str) -> Result<Vec<SeriesListItem>, TmdbError> {
        let response = self
            .client
            .get(format!("{BASE_URL}/search/tv"))
            .bearer_auth(&self.access_token)
            .query(&[("query", query), ("language", "en-US")])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| TmdbError::Api(e.to_string()))?;

        let data: super::models::SearchResponse = response.json().await?;
        Ok(data.results)
    }

    pub async fn get_series_details(&self, series_id: u64) -> Result<SeriesDetails, TmdbError> {
        let response = self
            .client
            .get(format!("{BASE_URL}/tv/{series_id}"))
            .bearer_auth(&self.access_token)
            .query(&[("language", "en-US")])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| TmdbError::Api(e.to_string()))?;

        let data: SeriesDetails = response.json().await?;
        Ok(data)
    }

    pub async fn get_popular_series(&self, page: u8) -> Result<Vec<SeriesListItem>, TmdbError> {
        let page_str = page.to_string();
        let response = self
            .client
            .get(format!("{BASE_URL}/tv/popular"))
            .bearer_auth(&self.access_token)
            .query(&[("language", "en-US"), ("page", &page_str)])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| TmdbError::Api(e.to_string()))?;

        let data: super::models::SearchResponse = response.json().await?;
        Ok(data.results)
    }

    /// Returns streaming/rent/buy availability for a given country code (e.g. "NO" for Norway).
    pub async fn get_watch_providers(
        &self,
        series_id: u64,
        country_code: &str,
    ) -> Result<Option<CountryProviders>, TmdbError> {
        let response = self
            .client
            .get(format!("{BASE_URL}/tv/{series_id}/watch/providers"))
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| TmdbError::Api(e.to_string()))?;

        let mut data: WatchProvidersResponse = response.json().await?;
        Ok(data.results.remove(country_code))
    }
}
