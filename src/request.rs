// Copyright 2023 Martin Pool

use serde::{Deserialize, Serialize};

/// The attributes of an AWS API request relevant to IAM policy evaluation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub action: String,
}
