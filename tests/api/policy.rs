// Copyright 2023 Martin Pool

use std::fs::read_to_string;

use eyre::Result;
use iamthat::principal::Principal;
use indoc::indoc;
use serde_json::json;

use iamthat::policy::*;
use iamthat::request::Request;

// TODO: Test more types of Action glob.

#[test]
fn load_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let _policy: Policy = serde_json::from_str(&json)?;
    Ok(())
}

#[test]
fn deny_unknown_fields_in_policy() {
    let err = serde_json::from_str::<Policy>(indoc! {r#"
        {
            "Version": "2000-01-01",
            "Color": "pink",
            "Statement": []
        }
        "#})
    .unwrap_err()
    .to_string();
    assert!(err.contains("unknown field `Color`"), "{err}");
}

#[test]
fn deny_unknown_fields_in_statement() {
    let err = serde_json::from_str::<Policy>(indoc! {r#"
        {
            "Version": "2000-01-01",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "*",
                    "Action": "*",
                    "Affect": "Allow"
                }
            ]
        }
        "#})
    .unwrap_err()
    .to_string();
    assert!(err.contains("unknown field `Affect`"), "{err}");
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
fn action_star_matches_in_resource_policy() -> Result<()> {
    let policy: Policy = serde_json::from_str(indoc! { r#"
        {
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "*",
                    "Action": "*",
                    "Principal": "*"
                }
            ]
        }
    "#})?;
    let request = Request {
        action: "s3:ListBuckets".to_owned(),
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };
    assert!(policy.allows(&request));
    Ok(())
}

#[test]
fn action_glob_case_insensitive_in_resource_policy() -> Result<()> {
    let policy: Policy = serde_json::from_value(json! {
        {
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "*",
                    "Action": "s3:*Bucket*",
                    "Principal": "*"
                }
            ]
        }
    })?;
    let request = Request {
        action: "S3:lISTbUCKETS".to_owned(),
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };
    assert!(policy.allows(&request));
    assert!(!policy.denies(&request));
    Ok(())
}

#[test]
fn action_matches_action_glob_in_resource_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let policy: Policy = serde_json::from_str(&json)?;
    let request = Request {
        action: "s3:ListBuckets".to_owned(),
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };
    assert!(policy.allows(&request));
    assert!(!policy.denies(&request));
    Ok(())
}

#[test]
fn action_does_not_match_resource_policy() -> Result<()> {
    let json = read_to_string("example/resource_policy/s3_list.json")?;
    let policy: Policy = serde_json::from_str(&json)?;
    let request = Request {
        action: "s3:CreateBucket".into(),
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };
    assert!(!policy.allows(&request));
    assert!(!policy.denies(&request));
    // No result on this specific policy; if it didn't match any policy
    // at all the result would be an implicit deny.
    Ok(())
}
