// Copyright 2023 Martin Pool

//! Tests for the `iamthat test` command.

use std::fs::read_to_string;
use std::str::FromStr;

use assert_fs::prelude::*;
use assert_fs::NamedTempFile;
use glob::glob;
use predicates::prelude::*;
use serde_json::{json, Value};

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
    outfile.assert(predicate::function(|s: &str| {
        Value::from_str(s).unwrap()
            == json! {
                    [
                      [
                        "Pass",
                        "Pass"
                      ]
                    ]
            }
    }));
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
    for path in glob("example/failing_tests/*_test.json")
        .expect("Glob testcases")
        .map(|r| r.expect("Read testcase name"))
    {
        println!("Test {path:?}");
        let actual = NamedTempFile::new("test_output.json").unwrap();
        let _out = run()
            .args(["test"])
            .arg(&path)
            .arg("--output")
            .arg(actual.path())
            .assert()
            .failure()
            .get_output();
        let expected_path = path
            .to_string_lossy()
            .replace("_test.json", "_expected.json");
        let expected_str = read_to_string(&expected_path).expect("read expected");
        println!("Expected:\n{expected_str}");
        let actual_str = read_to_string(actual.path()).expect("read actual");
        println!("Actual:\n{actual_str}");
        let expected_json = Value::from_str(&expected_str).expect("parse expected");
        let actual_json = Value::from_str(&actual_str).expect("parse actual");
        assert_eq!(actual_json, expected_json);
    }
}
