// Copyright 2023 Martin Pool

use std::fs::read_to_string;

use iamthat::policy;

fn main() {
    tracing_subscriber::fmt::init();

    let policy_json = read_to_string("example/s3_list.json").unwrap();
    let policy: policy::Policy = serde_json::from_str(&policy_json).unwrap();
    println!("{policy:#?}");
    println!();

    let request = policy::Request {
        action: "s3:ListBuckets".to_owned(),
    };
    println!("{:#?}", policy.eval(&request));
}
