use std::time::Duration;

use crate::{
  fitbit::{BodyType, FitbitClient, GetBodyRequest, StartDate, TimePeriod},
  state,
};
use anyhow::Result;
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use log::{info, warn};
use oauth2::TokenResponse;

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
    let sekrit = token.access_token().secret();

    let client = FitbitClient::new(|| Ok(sekrit.to_owned()));

    let result = client.get_body(GetBodyRequest {
      body_type: BodyType::Weight,
      start_date: StartDate::today(),
      time_period: TimePeriod::Max,
    })?;

    println!("{:?}", result);
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
