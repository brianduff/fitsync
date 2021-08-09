#![feature(proc_macro_hygiene, decl_macro, backtrace)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use config::Config;
use env_logger::Env;
use fitbit::FitbitClient;
use rocket::{fairing::AdHoc, Rocket};
use rocket_contrib::serve::StaticFiles;

mod api;
mod auth;
mod config;
mod fitbit;
mod runloop;

pub struct AppState {
  pub fitbit_client: FitbitClient,
  pub config: Config,
}

fn launch_browser(r: &Rocket) {
  let port = r.config().port;

  webbrowser::open(&format!("http://localhost:{}/", port)).unwrap();
}

fn main() -> Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let static_path = "static";

  let _scheduler = runloop::start();

  let config = Config::load()?;
  let fitbit_oauth = auth::OAuthClient::for_service("fitbit", &config.auth.fitbit)?;
  let fitbit_client = FitbitClient::new(fitbit_oauth);
  // let google_client  = auth::OAuthClient::for_service("google"", secrets)
  let app_state = AppState {
    config,
    fitbit_client,
  };

  rocket::ignite()
    .attach(AdHoc::on_launch("Launch", launch_browser))
    .manage(app_state)
    .mount("/api/", api::get_api_routes())
    .mount("/auth/", api::get_auth_routes())
    .mount("/", StaticFiles::from(static_path))
    .launch();

  Ok(())
}
