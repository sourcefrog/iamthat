// Copyright 2023 Martin Pool

//! Tests for the `iamthat test` command.

use assert_fs::prelude::PathAssert;
use assert_fs::NamedTempFile;
use indoc::indoc;
// use assert_cmd::prelude::*;
use glob::glob;
use predicates::prelude::*;

use super::*;

// A redundant test of just one example, just for an easy start.
#[test]
fn s3_basics() {
    let outfile = NamedTempFile::new("results.json").unwrap();
    run()
        .args(["test", "example/testcase/s3_basics.json"])
        .arg("--output")
        .arg(outfile.path())
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("Assertion passed"));
    outfile.assert(indoc! { "
        [
          [
            \"Pass\",
            \"Pass\"
          ]
        ]
"});
}

#[test]
fn all_example_testcases() {
    let paths = glob("example/testcase/*.json")
        .expect("Glob testcases")
        .map(|r| r.expect("Read testcase name"))
        .collect::<Vec<_>>();
    run().args(["test"]).args(paths).assert().success();
}

/// Tests that are expected to fail all do fail.

#[test]
fn failing_testcases() {
    for path in glob("example/failing_tests/*.json")
        .expect("Glob testcases")
        .map(|r| r.expect("Read testcase name"))
    {
        println!("Test {path:?}");
        run().args(["test"]).arg(path).assert().failure();
    }
}
