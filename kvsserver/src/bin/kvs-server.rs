#[macro_use] 
extern crate log;

use std::env;
use std::process::exit;
use std::net::SocketAddr;
use std::path::PathBuf;

use log::LevelFilter;

use kvsserver::*;
use clap::{App, AppSettings, Arg};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum KvsEngineType {
    kvs,
    sled,
}


fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let matches = App::new("kvs-server")
        .version("CARGO_PKG_VERSION")
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
        .arg(Arg::with_name("VERSION")
                .short("-V")
                .help("kvs-server version")
        )
        .get_matches();
    
    if matches.is_present("VERSION") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    info!("start kvs-server : {}", env!("CARGO_PKG_VERSION"));


    let bindaddr = matches.value_of("ADDR").unwrap_or("localhost:8900");
    info!("bind address at {}", bindaddr.parse::<SocketAddr>().expect("Error format of ipaddress"));
    
    let specified_engine = match matches.value_of("ENGINE") {
        Some("kvs") | None => KvsEngineType::kvs,
        Some("sled") => KvsEngineType::sled,
        _ =>  {
            eprintln!("unknown engine!");
            exit(1);
        }
    };

    let current_engine = match get_current_engine(env::current_dir()?)? {
        Some(engine) => {
            if (engine != specified_engine)
            {
                eprintln!("current_engine is different from the specified_engine");
                exit(1);
            }
            engine
        }
        None => {
            std::fs::write(env::current_dir()?.join("engine"), format!("{:?}", specified_engine));
            specified_engine
        }
    };


    match current_engine {
        KvsEngineType::kvs => {
            // start engine
            let engine = KvStore::open(env::current_dir()?)?;

            // Start server and listen
            KvsServer::new(engine).run(bindaddr)?;
            info!("start engine kvs successsful!");
        },
        KvsEngineType::sled => {
            let engine = SledKvStore::new(env::current_dir()?)?;
            KvsServer::new(engine).run(bindaddr)?;
            info!("start engine sled successsful!");
        },
    }
    

    Ok(())
}


fn get_current_engine<P : Into<PathBuf>> (path : P) -> Result<Option<KvsEngineType>> {
    let path = path.into();
    let engine = path.join("engine");

    if !engine.exists() {
        return Ok(None);
    }

    match std::fs::read_to_string(engine)?.as_ref() {
        "kvs" => Ok(Some(KvsEngineType::kvs)),
        "sled" => Ok(Some(KvsEngineType::sled)),
        _ => Ok(None)
    }
}

