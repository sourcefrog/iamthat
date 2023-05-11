// Copyright 2023 Martin Pool

//! A scenario contains some configured policies and some requests. Evaluating the
//! scenario yields an allow/deny result per request.

use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;
use serde::{Deserialize, Serialize};
use tracing::{info, trace};

use crate::effect::Effect;
use crate::json::FromJson;
use crate::policy::Policy;
use crate::request::Request;
use crate::Result;

/// A scenario file containing inline policies and requests.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Scenario {
    /// Resource policies by name.
    pub resource_policies: HashMap<String, Policy>,
}

/// A scenario potential with references to other files containing requests
/// and policies.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct ScenarioWithIncludes {
    /// Policy objects, as a map from name to the relative path containing
    /// the policy.
    pub resource_policy_files: HashMap<String, Utf8PathBuf>,
}

impl Scenario {
    /// Make a new empty scenario.
    pub fn new() -> Scenario {
        Scenario::default()
    }

    /// Load a policy, following any inclusions of policies or requests
    /// from other files.
    pub fn load(path: &Utf8Path) -> Result<Scenario> {
        let swi = ScenarioWithIncludes::from_json_file(path)?;
        info!(?swi);

        let mut resource_policies: HashMap<String, Policy> = HashMap::new();
        for (name, relpath) in swi.resource_policy_files {
            let path = path.parent().unwrap().join(relpath);
            info!(?name, ?path, "Load referenced resource policy file");
            let policy = Policy::from_json_file(&path)
                .wrap_err_with(|| format!("Load referenced policy from {path:?}"))?;
            resource_policies.insert(name, policy);
        }

        Ok(Scenario { resource_policies })
    }

    /// Evaluate a request against the policies and configuration of this
    /// scenario.
    pub fn eval(&self, request: &Request) -> Result<Effect> {
        // TODO: This should eventually implement the logic in
        // <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html>

        // TODO: Evaluate relevant identity policies .

        // First, does any policy deny the request?
        if self
            .resource_policies
            .iter()
            .any(|(_policy_type, policy)| policy.denies(request))
        {
            return Ok(Effect::Deny);
        }
        // TODO: Check for an Allow in various policy types in succession.
        // TODO: The interpretation of the resource-based policy depends on the
        // type of principal in the request.
        // TODO: Only check relevant resource policies.
        if self
            .resource_policies
            .values()
            .any(|policy| policy.allows(request))
        {
            return Ok(Effect::Allow);
        }

        trace!(?request, "No policy matched, so implicit deny");
        Ok(Effect::Deny)
    }

    pub fn add_resource_policy(&mut self, name: &str, policy: Policy) {
        assert!(
            !self.resource_policies.contains_key(name),
            "Resource policy {} already exists",
            name
        );
        self.resource_policies.insert(name.to_owned(), policy);
    }
}
