// Copyright 2023 Martin Pool

use std::fs::read_to_string;

use eyre::Result;
use indoc::indoc;

use iamthat::policy::{self, *};

#[test]
fn load_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let _policy: Policy = serde_json::from_str(&json)?;
    Ok(())
}

#[test]
fn deserialize_single_strings_abbreviate_lists() {
    let policy: Policy = serde_json::from_str(indoc! {r#"
        {
            "Version": "2000-01-01",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "*",
                    "Action": "s3:ListBuckets",
                    "NotPrincipal": {
                        "CanonicalUser": "012345"
                    }
                }
            ]
        }
        "#})
    .unwrap();
    assert_eq!(policy.statement[0].action, ["s3:ListBuckets"]);
    assert_eq!(policy.statement[0].resource, ["*"]);
    // TODO: Check NotPrincipal; doesn't seem right.
}

#[test]
fn action_matches_action_glob_in_resource_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let policy: Policy = serde_json::from_str(&json)?;
    let request = Request {
        action: "s3:ListBuckets".to_owned(),
    };
    assert_eq!(
        policy::eval_resource_policy(&policy, &request),
        Some(Effect::Allow)
    );
    Ok(())
}

#[test]
fn action_does_not_match_resource_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let policy: Policy = serde_json::from_str(&json)?;
    let request = Request {
        action: "s3:CreateBucket".into(),
    };
    assert_eq!(policy::eval_resource_policy(&policy, &request), None);
    // No result on this specific policy; if it didn't match any policy
    // at all the result would be an implicit deny.
    Ok(())
}
