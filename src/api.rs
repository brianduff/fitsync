use anyhow::Result;
use chrono::NaiveDate;

use crate::fitbit::{BodyType, GetBodyRequest, StartDate, TimePeriod};
use crate::AppState;
use log::info;
use rocket::response::Redirect;
use rocket::{Route, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

static FITBIT_SCOPES: &str =
  "activity heartrate location nutrition profile settings sleep social weight";

#[derive(Serialize, Deserialize, Debug)]
struct ServiceAuthState {
  has_token: bool,
  scopes: String,
  redirect_uri: String,
  client_id: String,
}

impl ServiceAuthState {
  fn create_fitbit(has_token: bool, client_id: String) -> Self {
    ServiceAuthState {
      has_token,
      scopes: FITBIT_SCOPES.to_owned(),
      redirect_uri: "http://localhost:8000/auth/fitbit".to_owned(), // FIXME: port
      client_id,
    }
  }

  fn create_google(has_token: bool, client_id: String) -> Self {
    ServiceAuthState {
      has_token,
      scopes: "".to_owned(),
      redirect_uri: "http://localhost:8000/".to_owned(),
      client_id,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthState {
  fitbit: ServiceAuthState,
  google: ServiceAuthState,
}

#[get("/authstate")]
fn authstate(state: State<AppState>) -> Result<Json<AuthState>> {
  let locked_oauth = state.fitbit_client.oauth.lock().unwrap();
  let has_fitbit_token = locked_oauth.has_secret();
  let has_google_token = false;

  Ok(Json(AuthState {
    fitbit: ServiceAuthState::create_fitbit(has_fitbit_token, locked_oauth.get_client_id()),
    google: ServiceAuthState::create_google(has_google_token, "".to_owned()),
  }))
}

#[get("/fitbit?<code>")]
fn fitbit_auth(code: Option<String>, state: State<AppState>) -> Result<Redirect> {
  info!("fitbit_auth started");
  match code {
    Some(code) => {
      {
        let mut oauth = state.fitbit_client.oauth.lock().expect("unable to lock");
        oauth.obtain_tokens(code)?;
      }

      // Force a sync to happen now.
      info!("fitbit_auth is requesting a sync");
      sync(state)?;

      info!("fitbit_auth done, redirecting");
      Ok(Redirect::to("/"))
    }
    None => {
      info!("fitbit_auth called with no code");
      Ok(Redirect::to("/"))
    }
  }
}

#[get("/sync")]
fn sync(state: State<AppState>) -> Result<()> {
  // let result = state.fitbit_client.get_body(GetBodyRequest {
  //   body_type: BodyType::Weight,
  //   start_date: StartDate::today(),
  //   time_period: TimePeriod::Max,
  // })?;
  let result = state.fitbit_client.get_body(GetBodyRequest {
    body_type: BodyType::Weight,
    start_date: StartDate::on_date(NaiveDate::from_ymd(2018, 8, 10)),
    time_period: TimePeriod::Max,
  })?;

  println!("{:?}", result);
  println!("{} records", result.body_weight.len());

  Ok(())
}

pub fn get_api_routes() -> Vec<Route> {
  routes![authstate, sync]
}

pub fn get_auth_routes() -> Vec<Route> {
  routes![fitbit_auth]
}
