use clap::{App, AppSettings, Arg , SubCommand};
use std::process::exit;
use kvs::{KvStore, Result, KvsError};
use std::env;

fn main() -> Result<()>{
    let matches = App::new(env!("CARGO_PKG_NAME"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .author(env!("CARGO_PKG_AUTHORS"))
                    .about(env!("CARGO_PKG_DESCRIPTION"))
                    .setting(AppSettings::DisableHelpSubcommand)
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .setting(AppSettings::VersionlessSubcommands)
                    .subcommand(
                        SubCommand::with_name("set")
                            .about("set Key-Value pair in the default KVStore")
                            .arg(Arg::with_name("KEY").help("a string key").required(true))
                            .arg(
                                Arg::with_name("VALUE")
                                    .help("a string value")
                                    .required(true)
                            )
                    )
                    .subcommand(
                        SubCommand::with_name("get")
                            .about("get Key-Value pair use key, if key is not exist, print the err info")
                            .arg(Arg::with_name("KEY").help("a string key").required(true))
                    )
                    .subcommand(
                        SubCommand::with_name("rm")
                            .about("remove Key-Value pair use key string")
                            .arg(Arg::with_name("KEY").help("a string key").required(true))
                    )
                    .get_matches();

    let path = env::current_dir()?;

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY is not specified!");
            let value = _matches.value_of("VALUE").expect("VALUE is not specified!");

            let mut kvs = KvStore::open(path)?;
            kvs.set(key.to_string(), value.to_string())?;
        },
        ("get", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY is not specified!");

            let mut kvs = KvStore::open(path)?;
            let value = kvs.get(key.to_string());
            match value {
                Ok(v) => {
                    if let Some(the_v) = v {
                        println!("{}", the_v);
                    } else {
                        println!("Key not found");
                    }
                },
                Err(_) => {
                    panic!()
                }
                
            }
        },
        ("rm", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY is not specified!");

            let mut kvs = KvStore::open(path)?;
            match kvs.remove(key.to_string()) {
                Ok(_) => {
                },
                Err(err) => {
                    match err {
                        KvsError::KeyNotFound => {
                            println!("Key not found");
                            exit(1);
                        },
                        _ => panic!()
                    }
                }
                
            }
        },
        _ => unreachable!()
    }

    Ok(())
}

