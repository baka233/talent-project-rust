#[macro_use] extern crate log;

pub use engine::{KvStore, KvsEngine};
pub use client::KvsClient;
pub use server::KvsServer;
pub use errors::{Result, KvsError};

mod common;
mod engine;
mod client;
mod server;
mod errors;
