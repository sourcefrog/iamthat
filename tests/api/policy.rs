// Copyright 2023 Martin Pool

use std::fs::read_to_string;

use eyre::Result;

use iamthat::policy::*;

#[test]
fn load_policy() -> Result<()> {
    let json = read_to_string("example/s3_list.json")?;
    let policy: Policy = serde_json::from_str(&json)?;
    Ok(())
}
