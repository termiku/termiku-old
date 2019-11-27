//! Configuration.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub env: Option<HashMap<String, String>>,
    pub font: FontConfig,
    pub shell: ShellConfig
}

#[derive(Deserialize, Debug, Clone)]
pub struct FontConfig {
    pub size: i16,
    pub path: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShellConfig {
    pub program: String,
    pub args: Vec<String>
}
