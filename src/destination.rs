use std::{
  collections::HashMap,
  fs::{read_to_string, File, OpenOptions},
  io::Write,
  path::PathBuf,
};

use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use csv::Writer;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::fitbit::{FitbitClient, TimeSeriesValue};

type DestinationId = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CsvFile {
  path: PathBuf,
}

impl DestinationAppender for CsvFile {
  fn append_data(&self, data: Vec<TimeSeriesValue>) -> Result<()> {
    let file = OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open(&self.path)?;

    let mut writer = Writer::from_writer(&file);
    for d in data {
      writer.serialize(&d)?;
    }

    Ok(())
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DestinationKind {
  CsvFile(CsvFile),
}

impl DestinationKind {
  fn get_appender(&self) -> Box<dyn DestinationAppender> {
    match self {
      Self::CsvFile(file) => Box::new(file.clone()),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Destination {
  id: DestinationId,
  kind: DestinationKind,
}

trait DestinationAppender {
  fn append_data(&self, data: Vec<TimeSeriesValue>) -> Result<()>;
}

impl Destination {
  pub fn append_data(&self, data: Vec<TimeSeriesValue>) -> Result<()> {
    self.kind.get_appender().append_data(data)
  }
}

#[derive(Serialize, Deserialize)]
pub struct DestinationConfig {
  pub destinations: Vec<Destination>,
}

impl DestinationConfig {
  fn new() -> Self {
    DestinationConfig {
      destinations: vec![Destination {
        id: "csv".to_owned(),
        kind: DestinationKind::CsvFile(CsvFile {
          path: PathBuf::from("basic.csv"),
        }),
      }],
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
    F: Fn(&Destination, &FitbitClient, Option<NaiveDateTime>) -> Result<()>,
  {
    for dest in self.config.destinations.iter() {
      let last_synced = if let Some(data) = self.cache.data.get(&dest.id) {
        data.last_synced
      } else {
        None
      };

      processor(dest, client, last_synced)?;
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
