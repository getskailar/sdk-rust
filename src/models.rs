//! Known model-ID constants.
//!
//! These are conveniences for the most common identifiers; the catalog is open
//! and changes over time, so the SDK accepts any `&str` model id. Call
//! [`Models::list`](crate::resources::models::Models::list) for the live set.

/// `claude-opus-4-8`
pub const CLAUDE_OPUS_4_8: &str = "claude-opus-4-8";
/// `claude-opus-4-7`
pub const CLAUDE_OPUS_4_7: &str = "claude-opus-4-7";
/// `claude-opus-4-6`
pub const CLAUDE_OPUS_4_6: &str = "claude-opus-4-6";
/// `claude-sonnet-4-6`
pub const CLAUDE_SONNET_4_6: &str = "claude-sonnet-4-6";
/// `claude-sonnet-4-5`
pub const CLAUDE_SONNET_4_5: &str = "claude-sonnet-4-5";
/// `claude-haiku-4-5`
pub const CLAUDE_HAIKU_4_5: &str = "claude-haiku-4-5";

/// `gpt-5.5`
pub const GPT_5_5: &str = "gpt-5.5";
/// `gpt-5.4`
pub const GPT_5_4: &str = "gpt-5.4";
/// `gpt-5.4-mini`
pub const GPT_5_4_MINI: &str = "gpt-5.4-mini";
/// `gpt-5.4-nano`
pub const GPT_5_4_NANO: &str = "gpt-5.4-nano";
/// `gpt-5.1`
pub const GPT_5_1: &str = "gpt-5.1";
/// `gpt-5`
pub const GPT_5: &str = "gpt-5";
/// `gpt-5-mini`
pub const GPT_5_MINI: &str = "gpt-5-mini";
/// `o3`
pub const O3: &str = "o3";
/// `o4-mini`
pub const O4_MINI: &str = "o4-mini";

/// `gemini-3.5-flash`
pub const GEMINI_3_5_FLASH: &str = "gemini-3.5-flash";
/// `gemini-3.1-pro-preview`
pub const GEMINI_3_1_PRO_PREVIEW: &str = "gemini-3.1-pro-preview";
/// `gemini-3-flash-preview`
pub const GEMINI_3_FLASH_PREVIEW: &str = "gemini-3-flash-preview";
/// `gemini-2.5-pro`
pub const GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
/// `gemini-2.5-flash`
pub const GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
/// `gemini-2.5-flash-lite`
pub const GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";

/// `deepseek-v4-pro`
pub const DEEPSEEK_V4_PRO: &str = "deepseek-v4-pro";
/// `deepseek-v4-flash`
pub const DEEPSEEK_V4_FLASH: &str = "deepseek-v4-flash";

/// `grok-4.3`
pub const GROK_4_3: &str = "grok-4.3";
/// `grok-4.20-reasoning`
pub const GROK_4_20_REASONING: &str = "grok-4.20-reasoning";
/// `grok-4.20-non-reasoning`
pub const GROK_4_20_NON_REASONING: &str = "grok-4.20-non-reasoning";
/// `grok-build-0.1`
pub const GROK_BUILD_0_1: &str = "grok-build-0.1";

/// `gpt-image-1`
pub const GPT_IMAGE_1: &str = "gpt-image-1";
