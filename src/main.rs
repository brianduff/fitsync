#![feature(proc_macro_hygiene, decl_macro, let_chains, backtrace)]

use std::process::Command;

#[macro_use]
extern crate rocket;

use env_logger::Env;
use rocket::{fairing::AdHoc, Rocket};
use rocket_contrib::serve::StaticFiles;

mod api;
mod config;
mod fitbit;
mod runloop;
mod state;

fn launch_browser(r: &Rocket) {
  let port = r.config().port;

  Command::new("open")
    .arg(format!("http://localhost:{}/", port))
    .output()
    .expect("Failed");
}

fn main() {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let static_path = "static";

  let _scheduler = runloop::start();

  rocket::ignite()
    .attach(AdHoc::on_launch("Launch", launch_browser))
    .mount("/api/", api::get_api_routes())
    .mount("/auth/", api::get_auth_routes())
    .mount("/", StaticFiles::from(static_path))
    .launch();
}
