// Copyright 2023 Martin Pool

use iamthat::principal::Principal;
use serde_json::json;
use tracing_test::traced_test;

use iamthat::json::FromJson;
use iamthat::policy::Policy;
use iamthat::scenario::Scenario;
use iamthat::Request;

#[test]
fn deny_overrides_allow() {
    let mut scenario = Scenario::new();
    scenario.add_resource_policy(
        "deny_and_allow",
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
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };

    assert!(scenario.eval(&request).unwrap().is_deny());
}

#[test]
#[traced_test]
fn scenario_with_no_policies_causes_implicit_deny() -> eyre::Result<()> {
    let scenario = Scenario::new();

    let request = Request {
        action: "aws-pca:IssueCertificate".to_string(),
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };

    assert!(scenario.eval(&request).unwrap().is_deny());

    Ok(())
}

#[test]
#[traced_test]
fn lack_of_match_is_implicit_deny() -> eyre::Result<()> {
    let mut scenario = Scenario::new();
    scenario.add_resource_policy(
        "allow_s3",
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
        principal: Principal::AWS {
            arn: "arn:aws:iam::111122223333:user/mateo".to_owned(),
        },
    };

    assert!(scenario.eval(&request).unwrap().is_deny());

    Ok(())
}
