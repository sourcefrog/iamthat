// Copyright 2023 Martin Pool

//! An IAM API request (or non-action permission), containing the action
//! name and relevant parameters and context.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The attributes of an AWS API request relevant to IAM policy evaluation.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Request {
    /// The AWS Action name, e.g. "s3:ListAllMyBuckets".
    pub action: String,
    // TODO: Resource, context, etc.
}
