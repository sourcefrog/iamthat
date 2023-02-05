// Copyright 2023 Martin Pool

use serde_json::json;
use tracing_test::traced_test;

use iamthat::json::FromJson;
use iamthat::policy::{Policy, PolicyType, Request};
use iamthat::policyset::PolicySet;

#[test]
fn deny_overrides_allow() {
    let mut policyset = PolicySet::new();
    policyset.add(
        PolicyType::Identity,
        Policy::from_json_value(json! {
        {
            "Version": "2000-01-01",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "s3:*",
                    "Action": "arn:aws:s3:::mybucket/*"
                },
                {
                    "Effect": "Deny",
                    "Resource": "s3:*",
                    "Action": "arn:aws:s3:::mybucket/*"
                }
            ]
        }
        })
        .unwrap(),
    );

    let request = Request {
        action: "s3:GetObject".to_string(),
    };

    assert!(policyset.eval(&request).is_deny());
}

#[test]
#[traced_test]
fn empty_policyset_is_implicit_deny() -> eyre::Result<()> {
    let policyset = PolicySet::new();

    let request = Request {
        action: "aws-pca:IssueCertificate".to_string(),
    };

    assert!(policyset.eval(&request).is_deny());

    Ok(())
}

#[test]
#[traced_test]
fn lack_of_match_is_implicit_deny() -> eyre::Result<()> {
    let mut policyset = PolicySet::new();
    policyset.add(
        PolicyType::Resource,
        Policy::from_json_value(json! {
        {
            "Version": "2000-01-01",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Resource": "s3:*",
                    "Action": "arn:aws:s3:::mybucket/*"
                }
            ]
        }
        })?,
    );

    let request = Request {
        action: "aws-pca:IssueCertificate".to_string(),
    };

    assert!(policyset.eval(&request).is_deny());

    Ok(())
}
