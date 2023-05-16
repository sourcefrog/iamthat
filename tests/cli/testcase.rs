// Copyright 2023 Martin Pool

//! Tests for the `iamthat test` command.

// use assert_cmd::prelude::*;
use glob::glob;
use predicates::prelude::*;

use super::*;

// A redundant test of just one example, just for an easy start.
#[test]
fn s3_basics() {
    run()
        .args(["test", "example/testcase/s3_basics.json"])
        .assert()
        .success()
        .stderr(
            predicate::str::contains("Assertion 0 passed")
                .and(predicate::str::contains("Assertion 1 passed")),
        );
}

#[test]
fn all_example_testcases() {
    let paths = glob("example/testcase/*.json")
        .expect("Glob testcases")
        .map(|r| r.expect("Read testcase name"))
        .collect::<Vec<_>>();
    run().args(["test"]).args(paths).assert().success().stderr(
        predicate::str::contains("Assertion 0 passed")
            .and(predicate::str::contains("Assertion 1 passed")),
    );
}
