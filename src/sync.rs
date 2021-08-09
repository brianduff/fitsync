use std::sync::{Mutex, MutexGuard};

use crate::{
  destination::{Destination, DestinationConfig, Destinations},
  fitbit::{BodyType, FitbitClient, GetBodyRequest, StartDate, TimePeriod},
};
use anyhow::Result;
use chrono::NaiveDate;

use log::{info, warn};

pub struct SyncSession<'a> {
  destinations: MutexGuard<'a, Destinations>,
  fitbit_client: &'a FitbitClient,
}

impl<'a> SyncSession<'a> {
  pub fn start(destinations: &'a Mutex<Destinations>, fitbit_client: &'a FitbitClient) -> Self {
    let locked = destinations.lock().unwrap();

    SyncSession {
      destinations: locked,
      fitbit_client,
    }
  }

  pub fn sync_all(&mut self) -> Result<()> {
    self.destinations.process(sync, self.fitbit_client)?;
    Ok(())
  }
}

fn sync(destination: &Destination, fitbit_client: &FitbitClient) -> Result<()> {
  let result = fitbit_client.get_body(GetBodyRequest {
    body_type: BodyType::Weight,
    start_date: StartDate::on_date(NaiveDate::from_ymd(2018, 8, 10)),
    time_period: TimePeriod::Max,
  })?;

  println!("{:?}", result);
  println!("{} records", result.body_weight.len());

  Ok(())
}
