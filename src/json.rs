// Copyright 2023 Martin Pool.

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
