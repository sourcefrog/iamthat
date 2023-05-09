// Copyright 2023 Martin Pool

//! Tests for `iamthat scenario`

use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use indoc::indoc;

use super::run;

#[test]
fn s3_list_scenario_succeeds() {
    let out_file = NamedTempFile::new("out.json").unwrap();
    run()
        .arg("scenario")
        .arg("example/scenario/s3_list.json")
        .arg("--output")
        .arg(out_file.path())
        .assert()
        .success();
    out_file.assert(indoc! { r#"
    [
      "Allow"
    ]
    "#});
}
