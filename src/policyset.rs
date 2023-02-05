// Copyright 2023 Martin Pool

use tracing::trace;

use crate::effect::Effect;
use crate::policy::{Policy, PolicyType, Request};

/// A set of policies relevant to some evaluations.
#[derive(Default)]
pub struct PolicySet {
    policies: Vec<(PolicyType, Policy)>,
}

impl PolicySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, policy_type: PolicyType, policy: Policy) {
        self.policies.push((policy_type, policy));
    }

    pub fn iter(&self) -> impl Iterator<Item = &(PolicyType, Policy)> {
        self.policies.iter()
    }

    pub fn eval(&self, request: &Request) -> Effect {
        // This should eventually implement the logic in
        // <https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_evaluation-logic.html>

        // First, does any policy deny the request?
        if self
            .policies
            .iter()
            .any(|(_policy_type, policy)| policy.denies(request))
        {
            return Effect::Deny;
        }
        // TODO: Check for an Allow in various policy types in succession.
        // TODO: The interpretation of the resource-based policy depends on the
        // type of principal in the request.
        if self.policies.iter().any(|(policy_type, policy)| {
            *policy_type == PolicyType::Resource && policy.allows(request)
        }) {
            return Effect::Allow;
        }

        trace!(?request, "No policy matched, so implicit deny");
        Effect::Deny
    }
}
