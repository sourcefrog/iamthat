// Copyright 2023 Martin Pool

//! IAM Policy documents.
//!

// Resources:
//
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html>
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html>

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::action::ActionGlob;
use crate::effect::Effect;
use crate::json::de_string_or_list;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Policy {
    pub version: Option<String>,
    pub id: Option<String>,
    pub statement: Vec<Statement>,
}

impl Policy {
    pub fn allows(&self, request: &Request) -> bool {
        self.statement
            .iter()
            .any(|statement| statement.allows(request))
    }

    pub fn denies(&self, request: &Request) -> bool {
        self.statement
            .iter()
            .any(|statement| statement.denies(request))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Statement {
    /// Statement id.
    pub sid: Option<String>,
    #[serde(flatten)]
    pub principal: Option<PrincipalOrNot>,

    pub effect: Effect,

    #[serde(deserialize_with = "de_string_or_list")]
    pub resource: Vec<String>, // TODO: Or NotResource

    #[serde(deserialize_with = "de_string_or_list")]
    pub action: Vec<String>, // TODO: Or NotAction

                             // TODO: Conditions
}

impl Statement {
    pub fn matches(&self, request: &Request) -> bool {
        // TODO: More conditions: principal, resource, action, conditions, etc.

        for statement_action in &self.action {
            if ActionGlob::from_str(statement_action)
                .map(|glob| glob.matches(&request.action))
                .unwrap_or(false)
            {
                debug!(?request, ?self, "action matches");
                return true;
            }
        }
        false
    }

    pub fn allows(&self, request: &Request) -> bool {
        self.effect.is_allow() && self.matches(request)
    }

    pub fn denies(&self, request: &Request) -> bool {
        self.effect.is_deny() && self.matches(request)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalOrNot {
    Principal(Vec<PrincipalMapEntry>),
    NotPrincipal(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalMapEntry {
    AWS(Vec<String>),
    Federated(Vec<String>),
    CanonicalUser(Vec<String>),
    Service(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub action: String,
}

// fn check_action_pattern(action_pattern: &str) {
//     if let Some(star) = action_pattern.find('*') {
//         if star != action_pattern.len() - 1 {
//             panic!("Star in {action_pattern:?} is not at the end");
//         }
//     }
// }

// See <https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum PolicyType {
    Resource,
    Identity,
    PermissionsBoundary,
    ServiceControl,
    Session,
}
