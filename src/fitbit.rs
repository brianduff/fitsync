use std::sync::Mutex;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::{blocking::Client, header::AUTHORIZATION};
use serde::{Deserialize, Serialize};
use strum_macros::ToString;

use crate::auth::OAuthClient;

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum ErrorType {
  ExpiredToken,
  #[serde(other)]
  Unknown,
}

#[derive(Deserialize, Debug)]
struct ApiError {
  #[serde(rename = "errorType")]
  error_type: ErrorType,
  message: String,
}

#[derive(Deserialize, Debug)]
struct GenericResponse {
  success: Option<bool>,
  errors: Option<Vec<ApiError>>,
  #[serde(rename = "body-weight")]
  body_weight: Option<Vec<TimeSeriesValue>>,
}

impl GenericResponse {
  fn has_expired_token(&self) -> bool {
    if let Some(ref errors) = self.errors {
      errors
        .iter()
        .any(|x| matches!(x.error_type, ErrorType::ExpiredToken))
    } else {
      false
    }
  }
}

pub trait TokenProvider {
  fn get_token(&self) -> Result<String>;
  fn refresh_token(&self) -> Result<String>;
}

pub struct FitbitClient {
  pub oauth: Mutex<OAuthClient>,
  http_client: Client,
}

impl FitbitClient {
  pub fn new(oauth: OAuthClient) -> Self {
    FitbitClient {
      oauth: Mutex::new(oauth),
      http_client: reqwest::blocking::Client::new(),
    }
  }

  fn make_request_with_secret(&self, url: &str, secret: &str) -> Result<GenericResponse> {
    let res = self
      .http_client
      .get(url)
      .header(AUTHORIZATION, format!("Bearer {}", secret))
      .header("Accept-Language", "en_US")
      .send()?;

    let text = res.text()?;
    serde_json::from_str(&text).with_context(|| format!("Couldn't parse: {}", text))
  }

  fn make_request(&self, url: String) -> Result<GenericResponse> {
    // TODO: Probably don't need to lock this for the whole duration of the request.
    let mut unlocked_oauth = self.oauth.lock().unwrap();
    let secret = unlocked_oauth.get_secret()?;
    let result = self.make_request_with_secret(&url, &secret)?;

    Ok(if !result.has_expired_token() {
      result
    } else {
      unlocked_oauth.refresh_tokens()?;
      let new_secret = unlocked_oauth.get_secret()?;
      self.make_request_with_secret(&url, &new_secret)?
    })
  }

  pub fn get_body(&self, request: GetBodyRequest) -> Result<TimeSeriesData> {
    let response = self.make_request(format!(
      "https://api.fitbit.com/1/user/-{}",
      request.to_url_path()
    ))?;

    if let Some(body_weight) = response.body_weight {
      Ok(TimeSeriesData { body_weight })
    } else {
      Err(anyhow!("Errors in response: {:?}", response))
    }
  }
}

mod date_format {
  use chrono::NaiveDate;
  use serde::{self, Deserialize, Deserializer, Serializer};
  const FORMAT: &str = "%Y-%m-%d";

  pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
  }

  pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let s = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&s)
  }
}

mod value_format {
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let v: Result<f32, D::Error> = s.parse().map_err(serde::de::Error::custom);
    v
  }

  pub fn serialize<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_f32(*value)
  }
}
