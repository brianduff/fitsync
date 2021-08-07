use anyhow::Result;
use chrono::NaiveDate;
use reqwest::{blocking::Client, header::AUTHORIZATION};
use serde::Deserialize;
use strum_macros::ToString;

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

#[derive(ToString)]
pub enum BodyType {
  Bmi,
  Fat,
  Weight,
}

impl ToUrlParameter for BodyType {
  fn to_url_parameter(&self) -> String {
    self.to_string().to_lowercase()
  }
}

#[derive(Debug, Clone, Copy)]
pub enum DateKind {
  Today,
  SpecificDate,
}

pub struct StartDate {
  date_kind: DateKind,
  date: Option<NaiveDate>,
}

impl StartDate {
  pub fn today() -> Self {
    StartDate {
      date_kind: DateKind::Today,
      date: None,
    }
  }

  pub fn on_date(date: NaiveDate) -> Self {
    StartDate {
      date_kind: DateKind::SpecificDate,
      date: Some(date),
    }
  }
}

impl ToUrlParameter for StartDate {
  fn to_url_parameter(&self) -> String {
    match self.date_kind {
      DateKind::Today => "today".to_owned(),
      DateKind::SpecificDate => self.date.unwrap().to_string(),
    }
  }
}

#[derive(ToString)]
pub enum TimePeriod {
  OneDay,
  SevenDays,
  ThirtyDays,
  OneWeek,
  OneMonth,
  ThreeMonths,
  SixMonths,
  OneYear,
  Max,
}

impl ToUrlParameter for TimePeriod {
  fn to_url_parameter(&self) -> String {
    match self {
      Self::OneDay => "1d",
      Self::SevenDays => "7d",
      Self::ThirtyDays => "30d",
      Self::OneWeek => "1w",
      Self::OneMonth => "1m",
      Self::ThreeMonths => "3m",
      Self::SixMonths => "6m",
      Self::OneYear => "1y",
      Self::Max => "max",
    }
    .to_owned()
  }
}

trait ToUrlParameter {
  fn to_url_parameter(&self) -> String;
}

trait ToUrlPath {
  fn to_url_path(&self) -> String;
}

pub struct GetBodyRequest {
  pub body_type: BodyType,
  pub start_date: StartDate,
  pub time_period: TimePeriod,
}

impl ToUrlPath for GetBodyRequest {
  fn to_url_path(&self) -> String {
    let body_type = self.body_type.to_url_parameter();
    let start_date = self.start_date.to_url_parameter();
    let time_period = self.time_period.to_url_parameter();

    format!(
      "/body/{}/date/{}/{}.json",
      body_type, start_date, time_period
    )
  }
}

pub struct FitbitClient<F>
where
  F: Fn() -> Result<String>,
{
  token_provider: F,
  http_client: Client,
}

impl<F> FitbitClient<F>
where
  F: Fn() -> Result<String>,
{
  pub fn new(token_provider: F) -> Self
  where
    F: Fn() -> Result<String>,
  {
    FitbitClient {
      token_provider,
      http_client: reqwest::blocking::Client::new(),
    }
  }

  pub fn get_body(&self, request: GetBodyRequest) -> Result<TimeSeriesData> {
    let url = format!("https://api.fitbit.com/1/user/-{}", request.to_url_path());

    let secret = (self.token_provider)()?;
    let res = self
      .http_client
      .get(url)
      .header(AUTHORIZATION, format!("Bearer {}", secret))
      .header("Accept-Language", "en_US")
      .send()?;

    deserialize(res.text()?)
  }
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
