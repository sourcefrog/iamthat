// Copyright 2023 Martin Pool.

use std::fmt;
use std::fs::read_to_string;
use std::path::Path;

use eyre::Context;
use serde::de::{self, Visitor};
use serde::Deserializer;

pub trait FromJson: Sized + serde::de::DeserializeOwned {
    fn from_json(json: &str) -> eyre::Result<Self>;

    fn from_json_file(path: &Path) -> eyre::Result<Self>;

    fn from_json_value(value: serde_json::Value) -> eyre::Result<Self> {
        serde_json::from_value(value).wrap_err("Failed to parse JSON")
    }
}

impl<T> FromJson for T
where
    T: Sized + serde::de::DeserializeOwned,
{
    fn from_json(json: &str) -> eyre::Result<Self> {
        serde_json::from_str(json).wrap_err("Failed to parse JSON")
    }

    fn from_json_file(path: &Path) -> eyre::Result<Self> {
        serde_json::from_str(
            read_to_string(path)
                .wrap_err("Failed to read file")?
                .as_str(),
        )
        .wrap_err("Failed to parse JSON")
    }
}

/// Deserialize either a single string or a list of strings.
///
/// Many places in the IAM grammar allow a list of one string to be
/// abbreviated as just a string.
pub(crate) fn de_string_or_list<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Like <https://serde.rs/string-or-struct.html>
    struct StringOrList();
    impl<'de> Visitor<'de> for StringOrList {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut v = Vec::new();
            while let Some(el) = seq.next_element()? {
                v.push(el)
            }
            Ok(v)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }
    }

    deserializer.deserialize_any(StringOrList())
}
