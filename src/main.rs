// Copyright 2023 Martin Pool

//! `iamthat` command line tool: simulate/evaluate AWS requests against IAM policies.

use std::fs::OpenOptions;
use std::io::{stderr, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use eyre::Context;
use iamthat::policy::Policy;
use iamthat::tag::Tag;
use iamthat::testcase::{AssertionResult, TestCase, TestCaseWithPaths};
use iamthat::user::User;
use schemars::schema_for;
use tracing::{info, trace};
use tracing_subscriber::prelude::*;

use iamthat::effect::Effect;
use iamthat::json::FromJson;
use iamthat::request::Request;
use iamthat::scenario::{Scenario, ScenarioWithPaths};
use iamthat::Result;

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
    /// Evaluate a request against against the policies
    /// in a scenario file, and print the results.
    Eval {
        /// The scenario file to evaluate
        #[arg(long, short, required = true)]
        scenario: Utf8PathBuf,

        /// Files containing JSON requests to evaluate
        #[arg(long, short, required = true)]
        request: Vec<Utf8PathBuf>,

        /// Write evaluation results as json to this file.
        #[arg(long, short)]
        output: Option<Utf8PathBuf>,
    },

    /// Emit json schemas for all file types defined by iamthat.
    JsonSchema {
        /// Write schemas into this directory.
        #[arg(long, short = 'o', required = true)]
        out_dir: Utf8PathBuf,
    },

    /// Evaluate all the requests in a testcase file against the policies
    /// in that scenario, and fail if the result is not as expected.
    Test {
        testcases: Vec<Utf8PathBuf>,
        /// Write json results to this file.
        #[arg(long, short)]
        output: Option<Utf8PathBuf>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum SchemaType {
    Request,
    TestCase,
}

fn main() -> eyre::Result<ExitCode> {
    let args = Args::parse();
    init_tracing(args.json_log.as_ref());

    match args.command {
        Command::Eval {
            output,
            scenario,
            request,
        } => {
            let scenario = Scenario::from_json_file(&scenario)
                .wrap_err_with(|| format!("failed to read scenario file {scenario:?}"))?;
            info!(?scenario);
            let requests = request
                .iter()
                .map(|p| {
                    Request::from_json_file(p)
                        .wrap_err_with(|| format!("Failed to read request file {p:?}"))
                })
                .collect::<eyre::Result<Vec<Request>>>()?;
            let effects = requests
                .into_iter()
                .map(|request| {
                    let effect = scenario.eval(&request);
                    info!(?request, ?effect);
                    effect
                })
                .collect::<Result<Vec<_>>>()?;
            if let Some(out_path) = output {
                let mut out = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&out_path)
                    .wrap_err_with(|| format!("failed to open output file {out_path:?}"))?;
                serde_json::to_writer_pretty(&mut out, &effects)?;
                writeln!(out)?;
                out.flush()?;
            }
            results_to_return_code(&effects)
        }
        Command::JsonSchema { out_dir } => {
            for (name, schema) in [
                ("policy", schema_for!(Policy)),
                ("request", schema_for!(Request)),
                ("scenario", schema_for!(ScenarioWithPaths)),
                ("tag", schema_for!(Tag)),
                ("testcase", schema_for!(TestCaseWithPaths)),
                ("user", schema_for!(User)),
            ] {
                let out_path = out_dir.join(format!("{}.json", name));
                let mut out = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&out_path)
                    .with_context(|| format!("Failed to open schema file {out_path:?}"))?;
                serde_json::to_writer_pretty(&mut out, &schema)?;
                writeln!(out)?;
            }
            info!("Schemas written to {}", out_dir.canonicalize_utf8().expect("Canonicalize out_path"));
            Ok(ExitCode::SUCCESS)
        }
        Command::Test {
            testcases: testcase_paths,
            output,
        } => {
            let results = testcase_paths
                .iter()
                .map(|path| TestCase::from_json_file(path))
                .collect::<Result<Vec<TestCase>>>()?
                .into_iter()
                .map(|tc| tc.eval())
                .collect::<Vec<Vec<AssertionResult>>>();
            if let Some(output_path) = output {
                let mut out = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(output_path)?;
                serde_json::to_writer_pretty(&mut out, &results)?;
                writeln!(out)?;
            }
            if results.iter().flatten().all(AssertionResult::is_pass) {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::FAILURE)
            }
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
