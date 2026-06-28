#![deny(clippy::all)]

mod agent;
mod config;
mod pool;
mod util;

use napi_derive::napi;

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
