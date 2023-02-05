// Copyright 2023 Martin Pool

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Copy, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
#[must_use]
pub enum Effect {
    Allow,
    Deny,
}

impl Effect {
    pub fn is_allow(&self) -> bool {
        *self == Effect::Allow
    }

    pub fn is_deny(&self) -> bool {
        *self == Effect::Deny
    }
}
