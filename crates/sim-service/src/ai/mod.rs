//! AI integration module for Agent Society Simulator
//!
//! Provides Gemini AI capabilities for simulation analysis while
//! maintaining SIM-001 compliance (no dependencies on product crates).
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! # Architecture
//!
//! - `gemini.rs`: Direct HTTP client for Google Gemini API
//! - `prompts.rs`: Simulation-aware prompt templates
//!
//! # Configuration
//!
//! Environment variables:
//! - `GEMINI_API_KEY` or `GOOGLE_AI_API_KEY` (required)
//! - `GEMINI_MODEL` (optional, default: gemini-2.5-flash)
//! - `GEMINI_ENDPOINT` (optional)
//! - `GEMINI_TIMEOUT_SECS` (optional, default: 30)

pub mod gemini;
/// prompts module
pub mod prompts;

// Re-export commonly used types
/// Re-export
pub use gemini::{
    Content, GeminiConfig, GeminiError, GeminiRequest, GeminiResponse, GenerationConfig,
    NotableEvent, Part, SimGeminiService, SimulationContext,
};

/// Re-export
pub use prompts::SimulationPromptBuilder;

use std::sync::Arc;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Shared AI service state for the simulation service
#[derive(Debug, Clone)]
pub struct AiState {
    /// The Gemini service (None if not configured)
    pub gemini: Option<Arc<SimGeminiService>>,

    /// Whether AI features are enabled
    pub enabled: bool,
}

impl AiState {
    /// Create a new AI state from environment
    pub fn from_env() -> Self {
        match SimGeminiService::from_env() {
            Ok(service) => Self {
                gemini: Some(Arc::new(service)),
                enabled: true,
            },
            Err(e) => {
                tracing::warn!("Gemini AI not configured: {}", e);
                Self {
                    gemini: None,
                    enabled: false,
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with explicit configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn with_config(config: GeminiConfig) -> Result<Self, GeminiError> {
        let service = SimGeminiService::with_config(config)?;
        Ok(Self {
            gemini: Some(Arc::new(service)),
            enabled: true,
        })
    }

    /// Create disabled state (for testing)
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            gemini: None,
            enabled: false,
        }
    }

    /// Check if AI is available
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.enabled && self.gemini.is_some()
    }
}

/// AI response with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiAnalysisResponse {
    /// The AI-generated insight
    pub insight: String,

    /// Model used for generation
    pub model: String,

    /// Whether this is a mock response (AI unavailable)
    pub is_mock: bool,

    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,

    /// Token usage (if available)
    pub token_usage: Option<TokenUsage>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// `TokenUsage` data
pub struct TokenUsage {
    /// prompt tokens
    pub prompt_tokens: i32,
    /// completion tokens
    pub completion_tokens: i32,
    /// total tokens
    pub total_tokens: i32,
}

impl AiAnalysisResponse {
    /// Create a mock response when AI is unavailable
    pub fn mock(insight: impl Into<String>) -> Self {
        Self {
            insight: insight.into(),
            model: "mock".to_string(),
            is_mock: true,
            generated_at: chrono::Utc::now(),
            token_usage: None,
        }
    }

    /// Create a real response from Gemini
    pub fn from_gemini(
        insight: impl Into<String>,
        model: impl Into<String>,
        usage: Option<gemini::UsageMetadata>,
    ) -> Self {
        Self {
            insight: insight.into(),
            model: model.into(),
            is_mock: false,
            generated_at: chrono::Utc::now(),
            token_usage: usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_token_count,
                completion_tokens: u.candidates_token_count,
                total_tokens: u.total_token_count,
            }),
        }
    }
}
