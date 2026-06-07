//! Utility response types.

use serde::{Deserialize, Serialize};

/// The response from `GET /v1/ping-key`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingKeyResponse {
    /// Always `"ok"` for a valid key.
    pub status: String,
    /// Identifier of the account that owns the key.
    pub user_id: String,
}
