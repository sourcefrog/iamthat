// Copyright 2023 Martin Pool

pub mod request;

use std::fs::{read_to_string, OpenOptions};
use std::io::{stderr, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use eyre::Context;
use iamthat::policyset::PolicySet;
use tracing::{error, info, trace};
use tracing_subscriber::prelude::*;

use iamthat::effect::Effect;
use iamthat::json::FromJson;
use iamthat::policy::{self, Policy, PolicyType};
use iamthat::request::Request;
use iamthat::scenario::Scenario;

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
        request_files: Vec<Utf8PathBuf>,

        #[arg(long = "resource_policy_file", short = 'R')]
        resource_policy_files: Vec<Utf8PathBuf>,
    },

    /// Evaluate all the requests in a scenario file against the policies
    /// in that scenario, and print the results.
    Scenario {
        /// The scenario file to evaluate
        scenario_file: Utf8PathBuf,

        /// Write evaluation results as json to this file.
        #[arg(long, short)]
        output: Option<Utf8PathBuf>,
    },
}

fn main() -> eyre::Result<ExitCode> {
    let args = Args::parse();
    init_tracing(args.json_log.as_ref());

    match args.command {
        Command::Eval {
            requests,
            request_files,
            resource_policy_files,
        } => {
            let mut policy_set = PolicySet::new();
            resource_policy_files
                .iter()
                .map(|p| {
                    Policy::from_json_file(p)
                        .wrap_err_with(|| format!("Failed to read policy file {p:?}"))
                })
                .collect::<eyre::Result<Vec<policy::Policy>>>()?
                .into_iter()
                .for_each(|policy| policy_set.add(PolicyType::Resource, policy));
            if policy_set.is_empty() {
                error!("No policies were provided");
                return Ok(ExitCode::FAILURE);
            }

            let req_jsons = request_files
                .iter()
                .map(|p| {
                    read_to_string(p).wrap_err_with(|| format!("Failed to read request file {p:?}"))
                })
                .collect::<eyre::Result<Vec<String>>>()?;
            let req_objects = req_jsons
                .into_iter()
                .chain(requests.into_iter())
                .map(|json| {
                    Request::from_json(&json)
                        .wrap_err_with(|| format!("Failed to parse request: {json}"))
                })
                .collect::<eyre::Result<Vec<Request>>>()?;

            let results = req_objects
                .into_iter()
                .map(|request| {
                    let effect = policy_set.eval(&request);
                    info!(?request, ?effect);
                    effect
                })
                .collect::<Vec<_>>();
            results_to_return_code(&results)
        }
        Command::Scenario {
            output,
            scenario_file,
        } => {
            let scenario = Scenario::load(&scenario_file)
                .wrap_err_with(|| format!("failed to read scenario file {scenario_file:?}"))?;
            info!(?scenario);
            let results = scenario.eval();
            info!(?results);
            if let Some(out_path) = output {
                let mut out = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&out_path)
                    .wrap_err_with(|| format!("failed to open output file {out_path:?}"))?;
                serde_json::to_writer_pretty(&mut out, &results)?;
                writeln!(out)?;
                out.flush()?;
            }
            results_to_return_code(&results)
        }
    }
}

fn results_to_return_code(results: &[Effect]) -> Result<ExitCode, eyre::ErrReport> {
    if results.iter().any(Effect::is_deny) {
        info!("Some requests were denied");
        Ok(ExitCode::FAILURE) // TODO: More specific for "success but denied"
    } else {
        info!("All requests were allowed");
        Ok(ExitCode::SUCCESS)
    }
}

fn init_tracing(json_log: Option<&PathBuf>) {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(stderr)
        .without_time();
    let json_file_layer = json_log
        .map(|p| {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(p)
                .unwrap()
        })
        .map(|f| tracing_subscriber::fmt::layer().with_writer(f).json());
    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(json_file_layer)
        .init();
    trace!("Tracing installed");
}
