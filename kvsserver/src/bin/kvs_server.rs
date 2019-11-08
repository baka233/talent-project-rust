use kvsserver::*;
use clap::{App, AppSettings, Arg};
use std::env;

fn main() -> Result<()> {
    let matches = App::new("kvs_client")
        .version("v1.0")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("ADDR")
                .help("server address like (HOST|IP):ADDR")
                .required(false)
        )
        .get_matches();

    let bindaddr = matches.value_of("ADDR").unwrap_or("localhost:8900");
    
    // start engine
    let engine = KvStore::open(env::current_dir()?)?;
    // Start server and listen
    KvsServer::new(engine).run(bindaddr)?;


    Ok(())
}

