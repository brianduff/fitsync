use std::{
  collections::HashMap,
  fs::{read_to_string, File},
  io::Write,
  path::PathBuf,
};

use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::fitbit::FitbitClient;

type DestinationId = String;

#[derive(Serialize, Deserialize)]
pub struct CsvFile {
  path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub enum DestinationKind {
  CsvFile(CsvFile),
}

#[derive(Serialize, Deserialize)]
pub struct Destination {
  id: DestinationId,
  kind: DestinationKind,
}

#[derive(Serialize, Deserialize)]
pub struct DestinationConfig {
  pub destinations: Vec<Destination>,
}

impl DestinationConfig {
  fn new() -> Self {
    DestinationConfig {
      destinations: vec![],
    }
  }
}

#[derive(Serialize, Deserialize)]
struct DestinationCacheData {
  pub last_synced: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
struct DestinationCache {
  data: HashMap<DestinationId, DestinationCacheData>,
}

impl DestinationCache {
  fn new() -> Self {
    DestinationCache {
      data: HashMap::new(),
    }
  }
}

pub struct Destinations {
  pub config: DestinationConfig,
  cache: DestinationCache,
  config_file: PathBuf,
  cache_file: PathBuf,
}

impl Destinations {
  pub fn load(project_dirs: &ProjectDirs) -> Result<Destinations> {
    let config_file = project_dirs.config_dir().join("destinations.json");
    let cache_file = project_dirs.cache_dir().join("destinations_cache.json");

    Ok(if !config_file.exists() {
      Destinations {
        config: DestinationConfig::new(),
        cache: DestinationCache::new(),
        config_file,
        cache_file,
      }
    } else {
      let cache = if cache_file.exists() {
        serde_json::from_str(&read_to_string(&cache_file)?)?
      } else {
        DestinationCache::new()
      };
      let config = serde_json::from_str(&read_to_string(&config_file)?)?;

      Destinations {
        config,
        cache,
        config_file,
        cache_file,
      }
    })
  }

  pub fn process<F>(&mut self, processor: F, client: &FitbitClient) -> Result<()>
  where
    F: Fn(&Destination, &FitbitClient) -> Result<()>,
  {
    for dest in self.config.destinations.iter() {
      processor(dest, client)?;
      let last_synced = Some(Utc::now().naive_local());
      if let Some(data) = self.cache.data.get_mut(&dest.id) {
        data.last_synced = last_synced;
      } else {
        self
          .cache
          .data
          .insert(dest.id.to_owned(), DestinationCacheData { last_synced });
      }
    }

    self.save_cache()?;

    Ok(())
  }

  fn save_cache(&self) -> Result<()> {
    let ser = serde_json::to_vec_pretty(&self.cache)?;

    std::fs::create_dir_all(self.cache_file.parent().unwrap())?;

    let mut file = File::create(&self.cache_file)?;
    file.write_all(&ser)?;

    Ok(())
  }
}
