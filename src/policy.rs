// Copyright 2023 Martin Pool

//! IAM Policy documents.
//!

// Resources:
//
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html>
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html>

use serde::{Deserialize, Serialize};
use tracing::debug;

// TODO: Various fields can be either a single value or list; deserialize
// them properly.

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Policy {
    pub version: String,
    pub id: Option<String>,
    pub statement: Vec<Statement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    /// Statement id.
    pub sid: Option<String>,
    #[serde(flatten, rename = "PascalCase")]
    pub principal: PrincipalOrNot,
    pub effect: Effect,
    pub resource: Vec<String>, // TODO: Or NotResource
    pub action: Vec<String>,   // TODO: Or NotAction
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Copy, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Effect {
    Allow,
    Deny,
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

fn action_matches(action_pattern: &str, action: &str) -> bool {
    if action_pattern == "*" {
        return true;
    }
    // TODO: Stars should be allowed at any point in the name,
    // but not in the service name, unless it's just '*' altogether.
    // TODO: Case-insensitive.
    if let Some(glob) = action_pattern.strip_suffix('*') {
        action.starts_with(glob)
    } else {
        action == action_pattern
    }
}

impl Policy {
    pub fn eval(&self, request: &Request) -> Option<Effect> {
        // Very approximate!

        // First, does anything deny?
        for statement in &self.statement {
            if statement.effect == Effect::Deny {
                if statement.matches(request) {
                    return Some(Effect::Deny);
                }
            }
        }

        if let Some(matched_policy) = self
            .statement
            .iter()
            .filter(|s| s.effect == Effect::Allow && s.matches(request))
            .next()
        {
            debug!(?matched_policy, "matches explicit allow");
            return Some(Effect::Allow);
        }

        for statement in &self.statement {
            if statement.effect == Effect::Deny {
                if statement.matches(request) {
                    return Some(Effect::Deny);
                }
            }
        }

        debug!(
            policy_id = self.id,
            ?request,
            "policy does not match request"
        );
        None
    }
}

impl Statement {
    pub fn matches(&self, request: &Request) -> bool {
        // TODO: More conditions

        for statement_action in &self.action {
            if action_matches(&statement_action, &request.action) {
                debug!(?request, ?self, "action matches");
                return true;
            }
        }
        false
    }
}
