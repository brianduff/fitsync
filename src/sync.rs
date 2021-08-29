use std::sync::{Mutex, MutexGuard};

use crate::{
  destination::{Destination, Destinations},
  fitbit::{BodyType, DateOrToday, FitbitClient, GetBodyRequest},
};
use anyhow::Result;
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};

use log::info;

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

fn end_date_for(start_date: NaiveDate) -> NaiveDate {
  let mut end_date = start_date + Duration::days(365);
  let today = Utc::now().naive_utc().date();
  if end_date > today {
    end_date = today;
  }

  end_date
}

fn sync(
  destination: &Destination,
  fitbit_client: &FitbitClient,
  last_synced: Option<NaiveDateTime>,
) -> Result<()> {
  info!("Syncing to destination {:?}", destination);

  let last_synced_date = last_synced.map(|dt| dt.date());
  let mut start_date = last_synced_date.unwrap_or_else(|| NaiveDate::from_ymd(2010, 1, 1));
  let mut end_date = end_date_for(start_date);

  let now = Utc::now().naive_utc().date();

  loop {
    let result = fitbit_client.get_body(GetBodyRequest::for_date_range(
      BodyType::Weight,
      DateOrToday::OnDate(start_date),
      end_date,
    ))?;
    destination.append_data(result.body_weight)?;

    if end_date == now {
      break;
    }

    start_date = end_date;
    end_date = end_date_for(start_date);
  }

  Ok(())
}
