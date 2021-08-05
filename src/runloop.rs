use std::time::Duration;

use crate::{fitbit, state};
use anyhow::Result;
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use log::{info, warn};
use oauth2::TokenResponse;
use reqwest::header::AUTHORIZATION;

pub fn synchronize() {
  info!("Beginning sync...");

  let result = synchronize_impl();
  if result.is_err() {
    warn!("Synchronize failed: {:?}", result.err());
  }

  info!("Sync completed");
}

fn synchronize_impl() -> Result<()> {
  let token = state::get_stored_token("fitbit");
  if let Some(token) = token {
    let url = "https://api.fitbit.com/1/user/-/body/weight/date/today/max.json";
    let sekrit = token.access_token().secret();

    let client = reqwest::blocking::Client::new();
    let res = client
      .get(url)
      .header(AUTHORIZATION, format!("Bearer {}", sekrit))
      .header("Accept-Language", "en_US")
      .send()?;

    println!("{:?}", fitbit::deserialize(res.text()?));
  } else {
    info!("No token for fitbit. Skipping sync.")
  }
  Ok(())
}

pub fn start() -> ScheduleHandle {
  info!("Starting run loop");

  synchronize();

  let mut scheduler = Scheduler::new();

  scheduler.every(15.minutes()).run(synchronize);
  scheduler.watch_thread(Duration::from_millis(10000))
}
