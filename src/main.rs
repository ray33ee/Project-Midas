mod host;
mod participant;
mod messages;
mod lua;

extern crate clap;

use clap::{crate_version, Arg, App, SubCommand};

fn main() {



    use std::net::SocketAddrV4;
    use std::str::FromStr;



    let app_matches = App::new("Midas")
        .version(crate_version!())
        .author("Will Cooper")
        .about("Distributed network based parallel computing system")
        .arg(Arg::with_name("socket address")
            .short("a")
            .long("address")
            .takes_value(true)
            .help("Socket address to host/connect to. Pleas specify Ip address and port number, such as '192.168.0.1:4000'.")
            .validator( |value|
                    match SocketAddrV4::from_str(value.as_str()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string())
                    }
                )
            .required(true))
        .subcommand(SubCommand::with_name("host")
            .arg(Arg::with_name("Lua script")
                .short("s")
                .long("script")
                .takes_value(true)
                .help("Lua script to run")
                .validator(|value|
                    if std::path::Path::new(value.as_str()).exists() {
                        Ok(())
                    }
                    else {
                        Err(format!("Lua script does not exist ({}).", value))
                    }
                )
                .required(true))
            .about("Executes Midas as the host"))
        .subcommand(SubCommand::with_name("participant")
            .about("Executes Midas as a participant"))
        .get_matches();

    let ip_address = app_matches.value_of("socket address").unwrap();

    match app_matches.subcommand() {
        ("host", host_matches) => {
            let mut host = host::Host::new(ip_address);

            host.add_code(host_matches.unwrap().value_of("Lua script").unwrap());

            loop {
                host.check_events();
            }
        },
        ("participant", _participant_matches) => {


            let mut participant = participant::Participant::new(ip_address);


            loop {
                participant.check_events();
            }

        },
        _ => unreachable!()
    };
}
