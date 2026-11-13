//! Gemini AI integration for simulation analysis
//!
//! Uses direct HTTP calls (reqwest) to maintain SIM-001 compliance.
//! This module MUST NOT depend on ai-adapter or engine crates.
//!
//! Environment variables:
//! - `GEMINI_API_KEY` or `GOOGLE_AI_API_KEY` (required)
//! - `GEMINI_MODEL` (optional, default: gemini-2.5-flash)
//! - `GEMINI_ENDPOINT` (optional, default: <https://generativelanguage.googleapis.com/v1beta>)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Errors that can occur when calling the Gemini API
#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("API key not found. Set GEMINI_API_KEY or GOOGLE_AI_API_KEY")]
    /// `ApiKeyNotFound`
    ApiKeyNotFound,

    #[error("HTTP request failed: {0}")]
    /// `RequestFailed`
    RequestFailed(#[from] reqwest::Error),

    #[error("No response from Gemini API")]
    /// `NoResponse`
    NoResponse,

    #[error("API error: {0}")]
    /// `ApiError`
    ApiError(String),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Gemini API request structure
#[derive(Debug, Serialize, Clone)]
pub struct GeminiRequest {
    /// contents
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// generation config
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// system instruction
    pub system_instruction: Option<SystemInstruction>,
}

#[derive(Debug, Serialize, Clone)]
/// Content data
pub struct Content {
    /// role
    pub role: String,
    /// parts
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
/// Part variants
pub enum Part {
    /// Text
    Text {
        /// text
        text: String,
    },
    /// `InlineData`
    InlineData {
        /// inline data
        inline_data: InlineData,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Part {
    /// text
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// inline data
    pub fn inline_data(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::InlineData {
            inline_data: InlineData {
                mime_type: mime_type.into(),
                data: data.into(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// `InlineData` data
pub struct InlineData {
    /// mime type
    pub mime_type: String,
    /// data
    pub data: String,
}

#[derive(Debug, Serialize, Clone)]
/// `GenerationConfig` data
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// temperature
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// max output tokens
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// top p
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// top k
    pub top_k: Option<i32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize, Clone)]
/// `SystemInstruction` data
pub struct SystemInstruction {
    /// parts
    pub parts: Vec<Part>,
}

/// Gemini API response structure
#[derive(Debug, Deserialize, Clone)]
pub struct GeminiResponse {
    /// candidates
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    /// usage metadata
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize, Clone)]
/// Candidate data
pub struct Candidate {
    /// content
    pub content: ContentResponse,
    #[serde(rename = "finishReason")]
    /// finish reason
    pub finish_reason: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize, Clone)]
/// `ContentResponse` data
pub struct ContentResponse {
    /// role
    pub role: String,
    /// parts
    pub parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
/// `PartResponse` variants
pub enum PartResponse {
    /// Text
    Text {
        /// text
        text: String,
    },
    /// `InlineData`
    InlineData {
        /// inline data
        inline_data: InlineData,
    },
}

#[derive(Debug, Deserialize, Clone)]
/// `UsageMetadata` data
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    /// prompt token count
    pub prompt_token_count: i32,
    #[serde(rename = "candidatesTokenCount")]
    /// candidates token count
    pub candidates_token_count: i32,
    #[serde(rename = "totalTokenCount")]
    /// total token count
    pub total_token_count: i32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Gemini service configuration
#[derive(Clone)]
pub struct GeminiConfig {
    /// api key
    pub api_key: String,
    /// model
    pub model: String,
    /// endpoint
    pub endpoint: String,
    /// timeout secs
    pub timeout_secs: u64,
}

impl std::fmt::Debug for GeminiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiConfig")
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("endpoint", &self.endpoint)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

impl GeminiConfig {
    /// Load configuration from environment variables
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self, GeminiError> {
        let api_key = std::env::var("FRESHCREDIT_GEMINI__API_KEY")
            .or_else(|_| std::env::var("GEMINI_API_KEY"))
            .or_else(|_| std::env::var("GOOGLE_AI_API_KEY"))
            .map_err(|_| GeminiError::ApiKeyNotFound)?;

        let model =
            std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());

        let endpoint = std::env::var("GEMINI_ENDPOINT")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());

        let timeout_secs = std::env::var("GEMINI_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        Ok(Self {
            api_key,
            model,
            endpoint,
            timeout_secs,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Gemini AI service for simulation analysis
///
/// Uses direct HTTP calls to maintain SIM-001 compliance.
/// Does NOT depend on ai-adapter or any product crates.
#[derive(Debug, Clone)]
pub struct SimGeminiService {
    config: GeminiConfig,
    client: Client,
}

impl SimGeminiService {
    /// Create a new Gemini service from environment configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self, GeminiError> {
        let config = GeminiConfig::from_env()?;
        Self::with_config(config)
    }

    /// Create a new Gemini service with explicit configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn with_config(config: GeminiConfig) -> Result<Self, GeminiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(GeminiError::RequestFailed)?;

        Ok(Self { config, client })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate content using Gemini API
    ///
    /// # Arguments
    /// * `contents` - Conversation contents
    /// * `system_instruction` - Optional system prompt
    /// * `config` - Optional generation configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn generate(
        &self,
        contents: Vec<Content>,
        system_instruction: Option<SystemInstruction>,
        config: Option<GenerationConfig>,
    ) -> Result<GeminiResponse, GeminiError> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.endpoint, self.config.model, self.config.api_key
        );

        let request = GeminiRequest {
            contents,
            generation_config: config,
            system_instruction,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(GeminiError::RequestFailed)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GeminiError::ApiError(format!(
                "HTTP {status}: {error_text}"
            )));
        }

        let gemini_response: GeminiResponse =
            response.json().await.map_err(GeminiError::RequestFailed)?;

        Ok(gemini_response)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simple text generation with a single prompt
    ///
    /// # Arguments
    /// * `prompt` - The user prompt
    /// * `system_prompt` - Optional system instruction
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn generate_text(
        &self,
        prompt: impl Into<String>,
        system_prompt: Option<impl Into<String>>,
    ) -> Result<String, GeminiError> {
        let contents = vec![Content {
            role: "user".to_string(),
            parts: vec![Part::text(prompt)],
        }];

        let system_instruction = system_prompt.map(|sp| SystemInstruction {
            parts: vec![Part::text(sp)],
        });

        let response = self
            .generate(
                contents,
                system_instruction,
                Some(GenerationConfig {
                    temperature: Some(0.7),
                    max_output_tokens: Some(2048),
                    top_p: None,
                    top_k: None,
                }),
            )
            .await?;

        response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| match p {
                PartResponse::Text { text } => Some(text.clone()),
                PartResponse::InlineData { .. } => None,
            })
            .ok_or(GeminiError::NoResponse)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate analysis with structured context
    ///
    /// # Arguments
    /// * `context` - Structured context for the analysis
    /// * `question` - Specific question to answer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn analyze_with_context(
        &self,
        context: &SimulationContext,
        question: impl Into<String>,
    ) -> Result<String, GeminiError> {
        let system_prompt = "You are an AI assistant analyzing an Agent Society Simulator. \
            Provide insights on emergent behaviors, trust dynamics, cooperation patterns, \
            and economic outcomes. Be concise but informative.";

        let prompt = format!(
            "Simulation Context:\n\
            - Current tick: {}\n\
            - Alive agents: {}\n\
            - Average energy: {:.2}\n\
            - Group count: {}\n\
            - DOCAS dimensions: {:?}\n\n\
            Recent notable events:\n{}\n\n\
            Question: {}\n\n\
            Provide a brief analysis based on the simulation context.",
            context.tick,
            context.alive_count,
            context.avg_energy,
            context.group_count,
            context.avg_docas,
            format_events(&context.notable_events),
            question.into()
        );

        self.generate_text(prompt, Some(system_prompt)).await
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if the service is healthy and configured
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    /// Get the configured model name
    #[must_use]
    pub fn model(&self) -> &str {
        &self.config.model
    }
}

/// Simulation context for AI analysis
#[derive(Debug, Clone, Default)]
pub struct SimulationContext {
    /// tick
    pub tick: u64,
    /// alive count
    pub alive_count: usize,
    /// avg energy
    pub avg_energy: f32,
    /// avg docas
    pub avg_docas: [f32; 5],
    /// group count
    pub group_count: usize,
    /// notable events
    pub notable_events: Vec<NotableEvent>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Notable event for AI context
#[derive(Debug, Clone)]
pub struct NotableEvent {
    /// event type
    pub event_type: String,
    /// agent id
    pub agent_id: Option<String>,
    /// description
    pub description: String,
}

fn format_events(events: &[NotableEvent]) -> String {
    if events.is_empty() {
        "None".to_string()
    } else {
        events
            .iter()
            .take(10)
            .map(|e| {
                format!(
                    "- [{}] {:?}: {}",
                    e.event_type,
                    e.agent_id.as_deref().unwrap_or("N/A"),
                    e.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_config_from_env() {
        // This test requires environment variables to be set
        // Run with: GEMINI_API_KEY=test cargo test test_gemini_config_from_env -- --ignored
    }

    #[test]
    fn test_part_creation() {
        let text_part = Part::text("Hello");
        match text_part {
            Part::Text { text } => assert_eq!(text, "Hello"),
            Part::InlineData { .. } => panic!("Expected text part"),
        }
    }

    #[test]
    fn test_sim_gemini_config_debug_redaction() {
        let config = GeminiConfig {
            api_key: "gemini_secret_api_key_xyz".to_string(),
            model: "gemini-2.5-flash".to_string(),
            endpoint: "https://generativelanguage.googleapis.com".to_string(),
            timeout_secs: 30,
        };
        let debug_str = format!("{config:?}");
        assert!(!debug_str.contains("gemini_secret_api_key_xyz"));
        assert!(debug_str.contains("[REDACTED]"));
    }
}
