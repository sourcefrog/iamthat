// Copyright 2023 Martin Pool

//! Test cases from JSON files, containing a scenario (containing policies etc)
//! and a request, and the expected result.

use camino::Utf8Path;

use camino::Utf8PathBuf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span, warn};

use crate::effect::Effect;
use crate::json::FromJson;
use crate::scenario::Scenario;
use crate::Request;
use crate::Result;

/// A test case containing a scenario (policies and resources) and a series of
/// assertions (requests and expected effects).
#[derive(Debug, Clone)]
pub struct TestCase {
    pub comment: Option<String>,
    pub scenario: Scenario,
    pub assertions: Vec<TestCaseAssertion>,
}

/// The result of evaluating one test assertion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssertionResult {
    Pass,
    Fail,
    Error(String),
}

/// A single assertion in a test case, containing a request and the expected
/// effect, and optionally a comment.
#[derive(Debug, Clone)]
pub struct TestCaseAssertion {
    pub comment: Option<String>,
    pub request: Request,
    pub expected: Effect,
}

impl TestCase {
    /// Load a test case and any referenced files.
    pub fn from_json_file(path: &Utf8Path) -> Result<TestCase> {
        let testcase_json = TestCaseWithPaths::from_json_file(path)?;
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

    /// Evaluate all the assertions in this test case.
    pub fn eval(&self) -> Vec<AssertionResult> {
        self.assertions
            .iter()
            .enumerate()
            .map(
                |(
                    i,
                    TestCaseAssertion {
                        request,
                        expected,
                        comment,
                    },
                )| {
                    let _span =
                        info_span!("Evaluate test assertion", ?request, ?expected, ?comment, ?i,)
                            .entered();
                    match self.scenario.eval(request) {
                        Err(err) => {
                            warn!(?err, "Error evaluating test assertion");
                            // Flatten to a string to avoid problems seriializing the error type.
                            AssertionResult::Error(err.to_string())
                        }
                        Ok(actual) if actual == *expected => {
                            info!("Assertion passed");
                            AssertionResult::Pass
                        }
                        Ok(_) => {
                            info!("Assertion failed");
                            AssertionResult::Fail
                        }
                    }
                },
            )
            .collect()
    }
}

impl AssertionResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, AssertionResult::Pass)
    }
}

/// A testcase consisting of a scenario referenced by path, and a series of assertions to evaluate.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TestCaseWithPaths {
    /// An optional comment explaining the test.
    pub comment: Option<String>,

    /// A path to a scenario file, relative to the testcase file.
    pub scenario: Utf8PathBuf,

    /// A series of requests and expected effects.
    pub assertions: Vec<AssertionWithRequestPath>,
}

/// An assertion in a testcase file, referencing a request file and giving the
/// expected effect.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct AssertionWithRequestPath {
    /// An optional comment explaining the assertion.
    pub comment: Option<String>,

    /// The path of the request file, relative to the testcase file.
    pub request: Utf8PathBuf,

    /// The expected effect.
    pub expected: Effect,
}
