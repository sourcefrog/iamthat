// Copyright 2023 Martin Pool

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A key-value tag.
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}
