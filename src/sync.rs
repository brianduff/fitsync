use std::sync::{Mutex, MutexGuard};

use crate::{
  destination::{Destination, DestinationConfig, Destinations},
  fitbit::{BodyType, FitbitClient, GetBodyRequest, GetWeightLogsRequest, StartDate, TimePeriod},
};
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};

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

fn sync(
  destination: &Destination,
  fitbit_client: &FitbitClient,
  last_synced: Option<NaiveDateTime>,
) -> Result<()> {
  info!("Syncing to destination {:?}", destination);

  let result = fitbit_client.get_body(GetBodyRequest {
    body_type: BodyType::Weight,
    start_date: StartDate::on_date(NaiveDate::from_ymd(2018, 8, 10)),
    time_period: TimePeriod::Max,
  })?;
  destination.append_data(result.body_weight)?;

  let result = fitbit_client.get_weight_logs(GetWeightLogsRequest {
    base_date: NaiveDate::from_ymd(2021, 8, 9),
    time_period: TimePeriod::OneMonth,
  })?;
  println!("Weight logs: {:?}", result);

  Ok(())
}