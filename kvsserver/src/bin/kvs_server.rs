use crate::client::KvsServer;
use clap::{App, AppSetting, Arg};
use std::env;

fn main() -> Result<()> {
    let matches = App::new("kvs_client")
        .version("v1.0")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSetting::SubcommandRequireElseHelp)
        .arg(Arg::with_name("ADDR")
                .help("server address like (HOST|IP):ADDR")
        )
        .get_matches();

    let bindaddr = matches.value_of("ADDR").unwrap_or("localhost:8900");
    
    // start engine
    let engine = KvStore::open(env::current_dir()?)?;
    // Start server and listen
    KvsServer::new(engine).run()?;


    Ok(())
}

