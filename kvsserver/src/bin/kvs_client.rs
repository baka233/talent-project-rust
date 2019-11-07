use crate::client::KvsClient;
use clap::*;

fn main() -> Result<()> {
    let matches = App::new("kvs_client")
        .version("v1.0")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSetting::SubcommandRequireElseHelp)
        .subcommand(
            SubCommand::with_name("set")
                .about("set a key-value pair")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("VALUE")
                            .help("he string value of key")
                            .required(true)
                )
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get the value from key")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("remove specified string key")
                .arg(Arg::with_name("KEY").help("a string key").required(true))
                .arg(Arg::with_name("ADDR")
                        .long("addr")
                        .help("server address like (HOST|IP):ADDR")
                )
        )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let kvs_client = KvsClient::new(addr)?;
            kvs_client.set(matches.value_of("KEY"), matches.value_of("VALUE"))?;
        },
        ("get", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let kvs_client = KvsClient::new(addr)?;
            match kvs_client.get(matches.value_of("KEY"))? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        },
        ("remove", Some(matches)) => {
            let addr = matches.value_of("ADDR").unwrap_or("localhost:8900");
            let kvs_client = KvsClient::new(addr)?;
            kvs_client.remove(matches.value_of("KEY"))?;           
        }
    };

    Ok(())
}
