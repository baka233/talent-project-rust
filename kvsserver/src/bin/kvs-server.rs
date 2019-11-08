#[macro_use] 
extern crate log;

use std::env;
use std::process::exit;
use std::net::SocketAddr;

use kvsserver::*;
use clap::{App, AppSettings, Arg};


fn main() -> Result<()> {
    log::set_logger(&LOGGER)
    let matches = App::new("kvs-server")
        .version("v1.0")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("ADDR")
                .long("--addr")
                .takes_value(true)
                .help("server address like (HOST|IP):ADDR")
        )
        .arg(Arg::with_name("ENGINE")
                .long("--engine")
                .takes_value(true)
                .help("select engine in (KvStore, SledKvStore)")
        )
        .get_matches();

    info!("start engine...");

    let bindaddr = matches.value_of("ADDR").unwrap_or("localhost:8900");
    info!("bind address at {}", bindaddr.parse::<SocketAddr>().expect("Error format of ipaddress"));

    match matches.value_of("ENGINE") {
        Some("KvStore") | None => {
            // start engine
            let engine = KvStore::open(env::current_dir()?)?;
            // Start server and listen
            KvsServer::new(engine).run(bindaddr)?;
            info!("start engine successsful!");
        },
        Some("SledKvStore") => {
            let engine = SledKvStore::new(env::current_dir()?)?;
            KvsServer::new(engine).run(bindaddr)?;
            info!("start engine successsful!");
        },
        Some(engine) => {
            error!("unknown engine {}!", engine);
            exit(1);
        }
    }
    

    Ok(())
}

