// Copyright 2023 Martin Pool

//! A scenario contains some configured policies and some requests. Evaluating the
//! scenario yields an allow/deny result per request.

use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::effect::Effect;
use crate::json::FromJson;
use crate::policy::{Policy, PolicyType};
use crate::policyset::PolicySet;
use crate::request::Request;
use crate::Result;

/// A scenario file containing inline policies and requests.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Scenario {
    // TODO: Probably a policyset rather than a map.
    /// Resource policies by name.
    pub resource_policies: HashMap<String, Policy>,
    // pub policies: Vec<Policy>,
    pub requests: Vec<Request>,
}

/// A scenario potential with references to other files containing requests
/// and policies.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct ScenarioWithIncludes {
    /// Policy objects, as a map from name to the relative path containing
    /// the policy.
    pub resource_policy_files: HashMap<String, Utf8PathBuf>,
    pub request_files: Vec<Utf8PathBuf>,
}

impl Scenario {
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

        let mut requests = Vec::new();
        for relpath in swi.request_files {
            let path = path.parent().unwrap().join(relpath);
            info!(?path, "Load referenced request file");
            requests
                .push(Request::from_json_file(&path).wrap_err_with(|| {
                    format!("failed to load referenced request from {path:?}")
                })?);
        }
        Ok(Scenario {
            resource_policies,
            requests,
        })
    }

    /// Evaluate each of the requests against the policies, and return
    /// a list of allow/deny results.
    pub fn eval(&self) -> Vec<Effect> {
        // TODO: Maybe merge Scenario with PolicySet?

        let mut policy_set = PolicySet::new();
        self.resource_policies
            .iter()
            .for_each(|(_name, policy)| policy_set.add(PolicyType::Resource, policy.clone()));
        self.requests
            .iter()
            .map(|request| policy_set.eval(request))
            .collect()
    }
}
