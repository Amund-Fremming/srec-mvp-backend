use reqwest::Client;
use thiserror::Error;

use crate::adapters::{
    db::models::Review,
    llm::models::{OpenAiMessage, OpenAiRequest, OpenAiResponse, SeriesRecommendations},
};

const MODEL: &str = "o4-mini";
const MAX_RETRIES: u8 = 3;

const SYSTEM_PROMPT: &str = "\
You are a TV series recommendation engine. Given a user's past ratings, suggest new series they would enjoy.\n\
If a user has previously rated a series that was recommended, treat that as a successful recommendation \
regardless of the score — your goal is to maximise recommendation quality, not predicted rating.\n\
\n\
You MUST respond with ONLY a valid JSON object — no markdown fences, no explanation, \
no text outside the JSON. The object must match this schema exactly:\n\
\n\
{\n\
  \"recommendations\": [\n\
    {\n\
      \"title\": \"string\",\n\
      \"genre\": \"string\",\n\
      \"confidence\": <integer between 0 and 100>\n\
    }\n\
  ],\n\
  \"taste_summary\": \"string — one sentence describing the user's taste profile\"\n\
}";

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned an error: {0}")]
    Api(String),
    #[error("Failed to parse LLM response after {attempts} attempt(s): {last_error}")]
    ParseFailed { attempts: u8, last_error: String },
}

#[derive(Clone)]
pub struct LlmAdapter {
    client: Client,
    api_key: String,
}

impl LlmAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Recommend series based on a user's reviews.  Each entry pairs a `Review`
    /// with the human-readable title of the series it refers to.
    pub async fn recommend(
        &self,
        reviews: &[(Review, String)],
    ) -> Result<SeriesRecommendations, LlmError> {
        let prompt = build_prompt(reviews);
        self.resilient_complete(prompt).await
    }

    /// Sends the prompt to the LLM and retries up to MAX_RETRIES times when the
    /// response cannot be deserialized. Each retry feeds the bad response back so
    /// the model can self-correct.
    async fn resilient_complete(
        &self,
        initial_prompt: String,
    ) -> Result<SeriesRecommendations, LlmError> {
        let mut messages = vec![OpenAiMessage {
            role: "user".into(),
            content: initial_prompt,
        }];
        let mut last_error = String::new();

        for attempt in 1..=MAX_RETRIES {
            let raw = self.call_api(&messages).await?;

            match serde_json::from_str::<SeriesRecommendations>(&raw) {
                Ok(parsed) => return Ok(parsed),
                Err(err) => {
                    last_error = err.to_string();
                    tracing::warn!(
                        attempt,
                        max = MAX_RETRIES,
                        error = %err,
                        "LLM response failed to deserialize; retrying"
                    );
                    // Feed the bad response back so the model can self-correct.
                    messages.push(OpenAiMessage {
                        role: "assistant".into(),
                        content: raw,
                    });
                    messages.push(OpenAiMessage {
                        role: "user".into(),
                        content: format!(
                            "Your previous response failed to parse. Error: {err}. \
                             Respond with ONLY the JSON object, no extra text."
                        ),
                    });
                }
            }
        }

        Err(LlmError::ParseFailed {
            attempts: MAX_RETRIES,
            last_error,
        })
    }

    async fn call_api(&self, messages: &[OpenAiMessage]) -> Result<String, LlmError> {
        let mut all_messages = vec![OpenAiMessage {
            role: "system".into(),
            content: SYSTEM_PROMPT.into(),
        }];
        all_messages.extend(messages.iter().map(|m| OpenAiMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        }));

        let body = OpenAiRequest {
            model: MODEL,
            messages: all_messages,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| LlmError::Api(e.to_string()))?;

        let raw_body = response.text().await?;

        let parsed: OpenAiResponse = serde_json::from_str(&raw_body)
            .map_err(|e| LlmError::Api(format!("failed to parse response: {e}")))?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| LlmError::Api("response contained no choices".into()))
    }
}

fn build_prompt(reviews: &[(Review, String)]) -> String {
    let entries: Vec<serde_json::Value> = reviews
        .iter()
        .map(|(r, title)| {
            serde_json::json!({
                "title": title,
                "rating": r.rating,
                "liked": r.liked,
                "disliked": r.disliked,
            })
        })
        .collect();

    let reviews_json = serde_json::to_string_pretty(&entries).unwrap_or_default();

    format!(
        "## User reviews\n{reviews_json}\n\n\
         Recommend 5 series the user would enjoy. Respond with ONLY the JSON object."
    )
}
