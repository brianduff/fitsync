use anyhow::Result;

use oauth2::reqwest::http_client;
use oauth2::{basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, TokenUrl};
use oauth2::{ClientSecret, RedirectUrl};
use rocket::response::Redirect;
use rocket::Route;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::thread;

use crate::config::Config;
use crate::runloop;
use crate::state;

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
  fn create_fitbit(has_token: bool, config: &Config) -> Self {
    ServiceAuthState {
      has_token,
      scopes: FITBIT_SCOPES.to_owned(),
      redirect_uri: "http://localhost:8000/auth/fitbit".to_owned(), // FIXME: port
      client_id: config.auth.fitbit.id.to_owned(),
    }
  }

  fn create_google(has_token: bool, config: &Config) -> Self {
    ServiceAuthState {
      has_token,
      scopes: "".to_owned(),
      redirect_uri: "http://localhost:8000/".to_owned(),
      client_id: config.auth.google.id.to_owned(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthState {
  fitbit: ServiceAuthState,
  google: ServiceAuthState,
}

#[get("/authstate")]
fn authstate() -> Result<Json<AuthState>> {
  let has_fitbit_token = state::get_stored_token("fitbit").is_some();
  let has_google_token = state::get_stored_token("google").is_some();
  let config = Config::load()?;

  Ok(Json(AuthState {
    fitbit: ServiceAuthState::create_fitbit(has_fitbit_token, &config),
    google: ServiceAuthState::create_google(has_google_token, &config),
  }))
}

#[get("/fitbit?<code>")]
fn fitbit_auth(code: Option<String>) -> Result<Redirect> {
  match code {
    Some(code) => {
      let config = Config::load()?;

      let client = BasicClient::new(
        ClientId::new(config.auth.fitbit.id),
        Some(ClientSecret::new(config.auth.fitbit.secret)),
        AuthUrl::new("https://www.fitbit.com/oauth2/authorize".to_string())?,
        Some(TokenUrl::new(
          "https://api.fitbit.com/oauth2/token".to_string(),
        )?),
      )
      .set_redirect_uri(RedirectUrl::new(
        "http://localhost:8000/auth/fitbit".to_string(),
      )?);
      let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .request(http_client)
        .unwrap();

      state::store_token("fitbit", token_result)?;

      // Force a sync to happen now.
      thread::spawn(runloop::synchronize);

      Ok(Redirect::to("/"))
    }
    None => {
      println!("Redirected. urm");
      Ok(Redirect::to("/"))
    }
  }
}

pub fn get_api_routes() -> Vec<Route> {
  routes![authstate]
}

pub fn get_auth_routes() -> Vec<Route> {
  routes![fitbit_auth]
}
