use anyhow::Result;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSeriesValue {
  #[serde(with = "date_format")]
  pub date_time: NaiveDate,
  #[serde(with = "value_format")]
  pub value: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TimeSeriesData {
  pub body_weight: Vec<TimeSeriesValue>,
}

pub fn deserialize<S: AsRef<str>>(s: S) -> Result<TimeSeriesData> {
  Ok(serde_json::from_str(s.as_ref())?)
}

mod date_format {
  use chrono::NaiveDate;
  use serde::{self, Deserialize, Deserializer};
  const FORMAT: &str = "%Y-%m-%d";

  pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
  }
}

mod value_format {
  use serde::{self, Deserialize, Deserializer};
  pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let v: Result<f32, D::Error> = s.parse().map_err(serde::de::Error::custom);
    v
  }
}
