//! Model catalog types.

use serde::{Deserialize, Serialize};

/// Capability flags for a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Whether the model supports SSE streaming.
    pub streaming: bool,
    /// Whether the model supports function/tool calling.
    pub tool_calls: bool,
    /// Whether the model accepts image inputs.
    pub vision: bool,
    /// Whether the model supports JSON-mode responses.
    pub json_mode: bool,
    /// Whether the model exposes a reasoning trace, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
}

/// Per-token pricing for a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Input price per million tokens.
    pub input_per_mtok: f64,
    /// Output price per million tokens.
    pub output_per_mtok: f64,
    /// ISO 4217 currency code (e.g. `"USD"`).
    pub currency: String,
}

/// A model as returned by [`list`](crate::resources::models::Models::list).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSummary {
    /// Model identifier.
    pub id: String,
    /// Object type; always `"model"`.
    pub object: String,
    /// Unix epoch seconds at registration.
    pub created: u64,
    /// Provider that owns the model.
    pub owned_by: String,
    /// Human-friendly display name.
    pub display_name: String,
    /// Maximum context window in tokens.
    pub context_window: u32,
    /// Maximum output tokens per request.
    pub max_output_tokens: u32,
    /// Capability flags.
    pub capabilities: ModelCapabilities,
    /// Pricing.
    pub pricing: ModelPricing,
    /// Lifecycle status (e.g. `"active"`, `"preview"`, `"deprecated"`).
    pub status: String,
}

/// A model detail card as returned by
/// [`retrieve`](crate::resources::models::Models::retrieve).
///
/// Carries every [`ModelSummary`] field, flattened, plus extended metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
    /// All summary fields, flattened into this struct.
    #[serde(flatten)]
    pub summary: ModelSummary,
    /// Long-form description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Supported input/output modalities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Modalities>,
    /// Request parameters the model honors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<Vec<String>>,
    /// Training-data knowledge cutoff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge_cutoff: Option<String>,
    /// Release date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,
    /// Link to provider documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    /// Known aliases that route to this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
}

/// Input/output modality lists for a [`Model`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modalities {
    /// Accepted input modalities (e.g. `"text"`, `"image"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<String>>,
    /// Produced output modalities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<String>>,
}

/// The envelope returned by `GET /v1/models`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelList {
    /// Object type; always `"list"`.
    pub object: String,
    /// The models.
    pub data: Vec<ModelSummary>,
}
