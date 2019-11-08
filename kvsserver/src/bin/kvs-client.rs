use kvsserver::*;
use clap::{App, AppSettings, Arg, SubCommand, ArgMatches};
use std::process::exit;

fn main() {
    let matches = App::new("kvs-client")
        .version("v1.0")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("set")
                .about("set a key-value pair")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("VALUE")
                            .help("he string value of key")
                            .required(true)
                )
                .arg(Arg::with_name("TIMES")
                        .long("times")
                        .takes_value(true)
                        .value_name("SET_TIMES")
                        .help("set the set times to test the performance")
                )
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .takes_value(true)
                        .value_name("IPADDR")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get the value from key")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .takes_value(true)
                        .value_name("IPADDR")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("remove specified string key")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .takes_value(true)
                        .value_name("IPADDR")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .get_matches();
    
    match run(matches) {
        Err(err) => {
            eprintln!("error occured : {}", err);
            exit(1);
        },
        _ => exit(0)
    }

}

fn run(matches : ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("set", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let times = matches.value_of("TIMES").unwrap_or("1").parse::<u64>().unwrap();
            let mut kvs_client = KvsClient::new(addr)?;
            let key = matches.value_of("KEY").expect("Key is not setted");
            let value = matches.value_of("VALUE").expect("Value is not setted");
            for i in 0..times {
                kvs_client.set(key.to_string(), value.to_string())?;
            }
        },
        ("get", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let mut kvs_client = KvsClient::new(addr)?;
            let key = matches.value_of("KEY").expect("Value is empty");
            match kvs_client.get(key.to_string())? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        },
        ("remove", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let mut kvs_client = KvsClient::new(addr)?;
            let key = matches.value_of("KEY").expect("Value is empty");
            kvs_client.remove(key.to_string())?;           
        },
        _ => unreachable!(),
    };

    Ok(())
}
