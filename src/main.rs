// Copyright 2023 Martin Pool

use std::fs::{read_to_string, OpenOptions};
use std::io::stderr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{trace, Level};
use tracing_subscriber::prelude::*;

use iamthat::policy;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Write a debug log in JSON format to this file
    #[arg(long, global = true)]
    json_log: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Eval,
}

fn main() {
    let args = Args::parse();
    init_tracing(args.json_log.as_ref());

    let policy_json = read_to_string("example/resource_policy/s3_list.json").unwrap();
    let policy: policy::Policy = serde_json::from_str(&policy_json).unwrap();
    println!("{policy:#?}");
    println!();

    let request = policy::Request {
        action: "s3:ListBuckets".to_owned(),
    };
    println!("{:#?}", policy::eval_resource_policy(&policy, &request));
}

fn init_tracing(json_log: Option<&PathBuf>) {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
        // .with_max_level(Level::TRACE)
        .without_time();
    let f = json_log.map(|p| {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(p)
            .unwrap()
    });
    let json_file_layer = f.map(|f| tracing_subscriber::fmt::layer().with_writer(f).json());
    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(json_file_layer)
        .init();
    trace!("Tracing installed");
}
