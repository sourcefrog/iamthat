// Copyright 2023 Martin Pool

pub mod action;
pub mod effect;
pub mod json;
pub mod policy;
pub mod principal;
pub mod request;
pub mod scenario;
pub mod tag;
pub mod testcase;
pub mod user;

pub use request::Request;

// TODO: Maybe a more specific and structured error type, to make this
// more usable as a library?
pub use eyre::{Error, Result};
