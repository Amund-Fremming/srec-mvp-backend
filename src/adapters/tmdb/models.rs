use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub page: u32,
    pub results: Vec<SeriesListItem>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SeriesListItem {
    pub id: u64,
    pub name: String,
    pub overview: String,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: u32,
    pub genre_ids: Vec<u32>,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub popularity: f64,
    pub poster_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SeriesDetails {
    pub id: u64,
    pub name: String,
    pub overview: String,
    pub first_air_date: Option<String>,
    pub last_air_date: Option<String>,
    pub status: String,
    pub vote_average: f64,
    pub vote_count: u32,
    pub popularity: f64,
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
    pub genres: Vec<Genre>,
    pub networks: Vec<Network>,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub poster_path: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub id: u32,
    pub name: String,
    pub origin_country: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WatchProvidersResponse {
    pub id: u64,
    pub results: HashMap<String, CountryProviders>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CountryProviders {
    pub link: Option<String>,
    #[serde(rename = "flatrate")]
    pub streaming: Option<Vec<Provider>>,
    pub rent: Option<Vec<Provider>>,
    pub buy: Option<Vec<Provider>>,
    pub free: Option<Vec<Provider>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Provider {
    pub provider_id: u32,
    pub provider_name: String,
    pub logo_path: Option<String>,
}
