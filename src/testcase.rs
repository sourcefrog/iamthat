// Copyright 2023 Martin Pool

//! Test cases from JSON files, containing a scenario (containing policies etc)
//! and a request, and the expected result.

use camino::Utf8Path;
use camino::Utf8PathBuf;
use eyre::{eyre, Context};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::effect::Effect;
use crate::json::FromJson;
use crate::scenario::Scenario;
use crate::Request;
use crate::Result;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub scenario: Scenario,
    pub assertions: Vec<TestCaseAssertion>,
}

impl TestCase {
    pub fn from_json_file(path: &Utf8Path) -> Result<TestCase> {
        let testcase_json = TestCaseJson::from_json_file(path)?;
        let dir = path.parent().unwrap();
        let scenario = Scenario::from_json_file(&dir.join(&testcase_json.scenario))?;
        let assertions = testcase_json
            .assertions
            .into_iter()
            .map(|assertion| {
                let request = Request::from_json_file(&dir.join(&assertion.request))?;
                Ok(TestCaseAssertion {
                    request,
                    expected: assertion.expected,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(TestCase {
            scenario,
            assertions,
        })
    }

    pub fn eval(&self) -> Vec<Result<()>> {
        self.assertions
            .iter()
            .enumerate()
            .map(|(i, TestCaseAssertion { request, expected })| {
                let actual = self
                    .scenario
                    .eval(request)
                    .with_context(|| format!("Failed to evaluate assertion {i}"))?;
                if actual == *expected {
                    info!(?actual, ?expected, ?request, "Assertion {i} passed");
                    Ok(())
                } else {
                    Err(eyre!(
                        "Expected {expected:?} but got {actual:?} for assertion {i}, request {request:?}",
                    ))
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TestCaseAssertion {
    pub request: Request,
    pub expected: Effect,
}

/// A testcase file containing a scenario, some requests, and expected results.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TestCaseJson {
    pub scenario: Utf8PathBuf,
    pub assertions: Vec<TestCaseAssertionJson>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TestCaseAssertionJson {
    pub request: Utf8PathBuf,
    pub expected: Effect,
}
