// Copyright 2023 Martin Pool

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tag::Tag;

/// An IAM user.
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct User {
    /// The user's name.
    pub user_name: String,
    /// The user's id.
    #[serde(default)]
    pub user_id: Option<String>,
    // /// The user's ARN.
    // // TODO: How should we treat this if unset?
    // #[serde(default)]
    // pub arn: Option<String>,
    /// The user's path, e.g. to group them under `/eng/`.
    #[serde(default = "slash")]
    pub path: String,

    // pub create_date: Option<String>,
    /// The user's tags.
    #[serde(default)]
    pub tags: Vec<Tag>,

    /// Named policies attached to this user.
    #[serde(default)]
    pub attached_policies: Vec<String>,
}

fn slash() -> String {
    "/".to_string()
}
