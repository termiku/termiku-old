//! Configuration.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub env: Option<HashMap<String, String>>,
}