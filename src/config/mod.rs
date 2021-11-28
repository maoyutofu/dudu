use crate::util;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub http: Http,
    pub publisher: Publisher,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    pub host: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Publisher {
    pub max_retry_count: u32,
    pub interval_time: u64,
    pub task_interval_time: u64,
}

impl Config {
    pub fn new() -> Self {
        match util::fs::read_to_str("config.toml") {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(c) => c,
                Err(e) => panic!("{}", e),
            },
            Err(e) => panic!("{}", e),
        }
    }
}
