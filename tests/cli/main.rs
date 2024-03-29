// Copyright 2023 Martin Pool

//! Tests for the iamthat command line.

mod eval;
mod generate_schemas;
mod testcase;

pub(crate) fn run() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("iamthat").expect("Failed to launch iamthat binary")
}
