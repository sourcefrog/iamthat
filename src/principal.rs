// Copyright 2023 Martin Pool

//! IAM principals

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One AWS principal.
///
/// For example this is the caller identity for a request.
// TODO: More options for federated, canonical user, etc?
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum Principal {
    /// A user, account, role, etc, identified by an ARN.
    ARN(String),
}

/// Matches a principal, or a list of principals, or states that they do not match.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalOrNot {
    Principal(Vec<PrincipalMapEntry>),
    NotPrincipal(Vec<String>), // TODO: Why not consistent?
}

/// Some principals, all of the same type, e.g. a list of AWS account ids.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalMapEntry {
    AWS(Vec<String>),
    Federated(Vec<String>),
    CanonicalUser(Vec<String>),
    Service(Vec<String>),
}
