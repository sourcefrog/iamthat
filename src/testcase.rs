// Copyright 2023 Martin Pool

//! Test cases from JSON files, containing a scenario (containing policies etc)
//! and a request, and the expected result.

use camino::Utf8Path;
use camino::Utf8PathBuf;
use eyre::{eyre, Context};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::trace;

use crate::effect::Effect;
use crate::json::FromJson;
use crate::scenario::Scenario;
use crate::Request;
use crate::Result;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub comment: Option<String>,
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
                    comment: assertion.comment,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(TestCase {
            scenario,
            assertions,
            comment: testcase_json.comment,
        })
    }

    pub fn eval(&self) -> Vec<Result<()>> {
        self.assertions
            .iter()
            .enumerate()
            .map(|(i, TestCaseAssertion { request, expected, comment })| {
                trace!(?request, ?expected, ?comment, "Evaluated assertion {i}");
                let actual = self
                    .scenario
                    .eval(request)
                    .with_context(|| format!("Failed to evaluate assertion {i}"))?;
                if actual == *expected {
                    info!(?actual, ?expected, ?request, "Assertion {i} passed");
                    Ok(())
                } else {
                    // TODO: Maybe distinguish that "we successfully evaluated the test and it failed"?
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
    pub comment: Option<String>,
    pub request: Request,
    pub expected: Effect,
}

/// A testcase file referencing a scenario file, and then a series of assertions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TestCaseJson {
    pub comment: Option<String>,
    pub scenario: Utf8PathBuf,
    pub assertions: Vec<TestCaseAssertionJson>,
}

/// An assertino in a testcase file, referencing a request file and giving the
/// expected effect.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TestCaseAssertionJson {
    pub comment: Option<String>,
    pub request: Utf8PathBuf,
    pub expected: Effect,
}
