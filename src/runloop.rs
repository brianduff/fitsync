use std::time::Duration;

use anyhow::Result;
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use log::info;

pub fn synchronize() {
  info!("Beginning sync...");

  info!("Sync completed");
}

fn synchronize_impl() -> Result<()> {
  // TODO: Send http request to self.
  Ok(())
}

pub fn start() -> ScheduleHandle {
  info!("Starting run loop");

  synchronize();

  let mut scheduler = Scheduler::new();

  scheduler.every(15.minutes()).run(synchronize);
  scheduler.watch_thread(Duration::from_millis(10000))
}
