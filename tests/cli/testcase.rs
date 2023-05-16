// Copyright 2023 Martin Pool

//! Tests for the `iamthat test` command.

use assert_cmd::prelude::*;
use predicates::prelude::*;

use super::*;

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
