use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesRecommendations {
    pub recommendations: Vec<Recommendation>,
    pub taste_summary: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Recommendation {
    pub title: String,
    pub genre: String,
    pub confidence: u8,
}

#[derive(Serialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct OpenAiRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<OpenAiMessage>,
}

#[derive(Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: MessageContent,
}

#[derive(Deserialize)]
pub struct MessageContent {
    pub content: String,
}
