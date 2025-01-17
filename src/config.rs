use std::fs::File;
use serde::{Deserialize, Serialize};
use crate::error::Error;

const DEFAULT_MODULES: [&'static str; 21] = [
    "cs2.exe",
    "client.dll",
    "engine2.dll",
    "schemasystem.dll",
    "animationsystem.dll",
    "rendersystemdx11.dll",
    "filesystem_stdio.dll",
    "inputsystem.dll",
    "materialsystem2.dll",
    "meshsystem.dll",
    "networksystem.dll",
    "panorama.dll",
    "panoramauiclient.dll",
    "resourcesystem.dll",
    "scenesystem.dll",
    "soundsystem.dll",
    "tier0.dll",
    "vphysics2.dll",
    "worldrenderer.dll",
    "matchmaking.dll",
    "server.dll"
];

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

    pub fn save(&self) -> Result<(), Error> {
        Ok(serde_json::to_writer_pretty(
            File::options()
                .create(true).read(true)
                .write(true).truncate(true)
                .open("config.json")?,
            self)?)
    }

    pub fn load() -> Result<Self, Error> {
        Ok(serde_json::from_reader(&File::options().read(true).open("config.json")?)?)
    }
}
