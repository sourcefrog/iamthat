// Copyright 2023 Martin Pool

use std::fs::read_to_string;
use std::io::stderr;

use tracing::{trace, Level};
use tracing_subscriber::prelude::*;

use iamthat::policy;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_max_level(Level::TRACE)
        .without_time()
        .finish()
        .init();
    trace!("Tracing installed");
}

fn main() {
    init_tracing();

    let policy_json = read_to_string("example/resource_policy/s3_list.json").unwrap();
    let policy: policy::Policy = serde_json::from_str(&policy_json).unwrap();
    println!("{policy:#?}");
    println!();

    let request = policy::Request {
        action: "s3:ListBuckets".to_owned(),
    };
    println!("{:#?}", policy::eval_resource_policy(&policy, &request));
}
