// Copyright 2023 Martin Pool

//! IAM Policy documents.
//!

// Resources:
//
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html>
// * <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html>

use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::action::ActionGlob;
use crate::effect::Effect;
use crate::json::de_string_or_list;
use crate::request::Request;

/// An IAM policy document, containing some statements.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Policy {
    /// The version of the IAM grammar, "2008-10-17" or "2012-10-17".
    pub version: Option<String>,
    /// A user-supplied id for the policy. Some services have special
    /// constraints on the id.
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

/// One statement in a policy, stating that requests matching some conditions
/// should be either allowed or denied.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Statement {
    /// Statement id.
    ///
    /// "For IAM policies, basic alphanumeric characters (A-Z,a-z,0-9) are the only allowed characters
    /// in the Sid value. Other AWS services that support resource policies may have other
    /// requirements for the Sid value. For example, some services require this value to be
    /// unique within an AWS account, and some services allow additional characters such as
    /// spaces in the Sid value."
    pub sid: Option<String>,

    /// The principal to which this statement applies.
    #[serde(flatten)]
    pub principal: Option<PrincipalOrNot>,

    /// The effect of this statement: allow or deny.
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
            match ActionGlob::from_str(statement_action) {
                Ok(glob) => {
                    if glob.matches(&request.action) {
                        debug!(?request, ?self, "action matches");
                        return true;
                    }
                }
                Err(e) => {
                    warn!(?statement_action, ?e, "action glob parse error");
                }
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

/// Matches a principal, or a list of principals, or states that they do not match.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalOrNot {
    Principal(Vec<PrincipalMapEntry>),
    NotPrincipal(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalMapEntry {
    AWS(Vec<String>),
    Federated(Vec<String>),
    CanonicalUser(Vec<String>),
    Service(Vec<String>),
}

// See <https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
pub enum PolicyType {
    Resource,
    Identity,
    PermissionsBoundary,
    ServiceControl,
    Session,
}
