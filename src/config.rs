use std::fs::File;
use serde::{Deserialize, Serialize};
use crate::{platform::DEFAULT_MODULES, error::Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub dump_modules: bool,
    pub modules: Option<Vec<String>>
}

impl Config {
    pub fn new() -> Self {
        Self {
            dump_modules: true,
            modules: Some(DEFAULT_MODULES.iter().map(|&s| s.to_string()).collect())
        }
    }

    pub fn save(&self) -> Result<()> {
        Ok(serde_json::to_writer_pretty(
            File::options()
                .create(true).read(true)
                .write(true).truncate(true)
                .open("config.json")?,
            self)?)
    }

    pub fn load() -> Result<Self> {
        Ok(serde_json::from_reader(&File::options().read(true).open("config.json")?)?)
    }
}
