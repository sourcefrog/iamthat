// Copyright 2023 Martin Pool

use std::fs::{read_to_string, OpenOptions};
use std::io::stderr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use eyre::Context;
use tracing::{info, trace, Level};
use tracing_subscriber::prelude::*;

use iamthat::json::{FromJson, FromJsonFile};
use iamthat::policy::{self, Request};

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
    /// Evaluate whether a request is allowed by a set of policies.
    Eval {
        /// The request to evaluate, as a JSON string
        #[arg(long = "request")]
        requests: Vec<String>,

        /// Files containing JSON requests to evaluate
        #[arg(long = "request_file")]
        request_files: Vec<PathBuf>,
    },
}

fn main() -> eyre::Result<ExitCode> {
    let args = Args::parse();
    init_tracing(args.json_log.as_ref());

    let policy_json = read_to_string("example/resource_policy/s3_list.json").unwrap();
    let policy: policy::Policy = serde_json::from_str(&policy_json).unwrap();
    println!("{policy:#?}");
    println!();

    match args.command {
        Command::Eval {
            requests,
            request_files,
        } => {
            let req_jsons = request_files
                .iter()
                .map(|p| read_to_string(p).wrap_err("Failed to read request file"))
                .collect::<eyre::Result<Vec<String>>>()?;
            let req_objects = req_jsons
                .into_iter()
                .chain(requests.into_iter())
                .map(|json| Request::from_json(&json).wrap_err("Failed to parse request"))
                .collect::<eyre::Result<Vec<Request>>>()?;
            let results = req_objects
                .into_iter()
                .map(|req| {
                    let result =
                        policy::eval_resource_policy(&policy, &req).unwrap_or(policy::Effect::Deny);
                    info!(?req, ?result);
                    result
                })
                .collect::<Vec<_>>();
            if results.iter().any(|effect| *effect == policy::Effect::Deny) {
                info!("Some requests were denied");
                Ok(ExitCode::FAILURE) // TODO: More specific for "success but denied"
            } else {
                info!("All requests were allowed");
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn init_tracing(json_log: Option<&PathBuf>) {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
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
