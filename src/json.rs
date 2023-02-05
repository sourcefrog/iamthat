// Copyright 2023 Martin Pool.

use std::fs::read_to_string;
use std::path::Path;

use eyre::Context;

pub trait FromJson<'de>
where
    Self: Sized,
{
    fn from_json(json: &'de str) -> eyre::Result<Self>;
}

impl<'de, T> FromJson<'de> for T
where
    T: serde::Deserialize<'de>,
{
    fn from_json(json: &'de str) -> eyre::Result<Self> {
        serde_json::from_str(json).wrap_err("Failed to parse JSON")
    }
}

pub trait FromJsonFile
where
    Self: Sized + serde::de::DeserializeOwned,
{
    fn from_json_file(path: &Path) -> eyre::Result<Self> {
        serde_json::from_str(
            read_to_string(path)
                .wrap_err("Failed to read file")?
                .as_str(),
        )
        .wrap_err("Failed to parse JSON")
    }
}
