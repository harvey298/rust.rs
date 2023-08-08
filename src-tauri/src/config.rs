use std::{fs, io::Write};

use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub fcm_credentials: String,
    pub expo_token: String,
    pub steam_token: String
}

impl Config {
    pub fn save(&self, path: &str) -> Result<()> {

        let cfg = toml::to_string_pretty(self)?;

        let mut file = fs::OpenOptions::new().create(true).write(true).append(false).open(path)?;
        file.write_all(cfg.as_bytes())?;

        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        let cfg_txt = fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&cfg_txt)?;
        Ok(cfg)
    }
}