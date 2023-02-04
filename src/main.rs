// Copyright 2023 Martin Pool

use std::fs::{read_to_string, OpenOptions};
use std::io::stderr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::Context;
use tracing::{trace, Level};
use tracing_subscriber::prelude::*;

use iamthat::json::FromJson;
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
        #[arg(long)]
        request: String,
    },
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    init_tracing(args.json_log.as_ref());

    match args.command {
        Command::Eval { request } => eval(&request),
    }
}

fn eval(request: &String) -> eyre::Result<()> {
    let policy_json = read_to_string("example/resource_policy/s3_list.json").unwrap();
    let policy: policy::Policy = serde_json::from_str(&policy_json).unwrap();
    println!("{policy:#?}");
    println!();

    let request = Request::from_json(request).wrap_err("Failed to parse request")?;
    println!("{:#?}", policy::eval_resource_policy(&policy, &request));
    Ok(())
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
