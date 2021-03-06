use std::path::Path;

use crate::auth::ServiceClient;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthConfig {
  pub fitbit: ServiceClient,
  pub google: ServiceClient,
}

#[derive(Deserialize)]
pub struct Config {
  pub auth: AuthConfig,
}

impl Config {
  pub fn load() -> Result<Self> {
    let config_file = Path::new("config.json");
    anyhow::ensure!(
      config_file.exists(),
      "You must create a config.json file containing auth client ids and secrets"
    );
    let text = std::fs::read_to_string(config_file)?;
    Ok(serde_json::from_str(&text)?)
  }
}
