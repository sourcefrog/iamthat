// Copyright 2023 Martin Pool

use std::str::FromStr;

use eyre::{bail, eyre, WrapErr};
use regex::Regex;

/// Some kind of "Action" pattern: a wildcard, a literal, or a glob.
pub enum ActionGlob {
    Star,
    Literal(String),
    Pattern(Regex),
}

impl FromStr for ActionGlob {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(ActionGlob::Star);
        }
        let (service, action) = s
            .split_once(':')
            .ok_or_else(|| eyre!("no colon in action pattern"))?;
        let service_re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
        if !service_re.is_match(service) {
            bail!("invalid service {service:?}");
        }
        let valid_action = Regex::new(r"^[a-zA-Z0-9*]+$").unwrap();
        if !valid_action.is_match(action) {
            bail!("invalid action glob {action:?}");
        } else if action.contains('*') {
            let action_re_str = format!("^(?i){service}:{}$", action.replace('*', ".*"));
            let action_re = Regex::new(&action_re_str).wrap_err_with(|| {
                "failed to compile action regexp {action_re_str:?} from {action:?}"
            })?;
            Ok(ActionGlob::Pattern(action_re))
        } else {
            Ok(ActionGlob::Literal(s.to_owned()))
        }
    }
}

impl ActionGlob {
    pub fn matches(&self, action: &str) -> bool {
        match self {
            ActionGlob::Star => true,
            ActionGlob::Literal(a) => a.eq_ignore_ascii_case(action),
            ActionGlob::Pattern(re) => re.is_match(action),
        }
    }
}
