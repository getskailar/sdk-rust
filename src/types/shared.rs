//! Types shared across multiple resources.

use serde::{Deserialize, Serialize};

/// Token accounting for a completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens consumed by the prompt.
    pub prompt_tokens: u32,
    /// Tokens produced in the completion.
    pub completion_tokens: u32,
    /// Sum of prompt and completion tokens.
    pub total_tokens: u32,
}

/// A function-calling tool definition, OpenAI-compatible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// Discriminator; always `"function"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The function the model may call.
    pub function: FunctionDef,
}

impl Tool {
    /// Builds a `function`-typed tool from a name and JSON-Schema parameters.
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Tool {
            kind: "function".to_owned(),
            function: FunctionDef {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
            },
        }
    }
}

/// The function half of a [`Tool`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Function name the model uses to call it.
    pub name: String,
    /// Natural-language description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema describing the function arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// A model's request to invoke a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Identifier echoed back in the corresponding `tool` message.
    pub id: String,
    /// Discriminator; always `"function"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The function invocation.
    pub function: FunctionCall,
}

/// The function invocation carried by a [`ToolCall`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function to call.
    pub name: String,
    /// JSON-encoded arguments string (not a parsed object).
    pub arguments: String,
}

/// Controls whether and how the model may call tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// One of `"auto"`, `"none"`, or `"required"`.
    Mode(String),
    /// Force a specific named function.
    Named(NamedToolChoice),
}

impl ToolChoice {
    /// Let the model decide whether to call a tool.
    pub fn auto() -> Self {
        ToolChoice::Mode("auto".to_owned())
    }

    /// Forbid tool calls for this request.
    pub fn none() -> Self {
        ToolChoice::Mode("none".to_owned())
    }

    /// Require the model to call at least one tool.
    pub fn required() -> Self {
        ToolChoice::Mode("required".to_owned())
    }

    /// Force the model to call the named function.
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Named(NamedToolChoice {
            kind: "function".to_owned(),
            function: NamedFunction { name: name.into() },
        })
    }
}

/// A [`ToolChoice`] that pins a specific function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedToolChoice {
    /// Discriminator; always `"function"`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The function to force.
    pub function: NamedFunction,
}

/// The name wrapper inside a [`NamedToolChoice`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedFunction {
    /// Function name to force.
    pub name: String,
}
