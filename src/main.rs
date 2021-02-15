mod operand;
mod instructions;
mod host;
mod participant;
mod messages;

extern crate clap;

use clap::{crate_version, Arg, App};
use crate::instructions::{get_instructions, Instructions, get_constants};

fn main() {

    use std::net::SocketAddrV4;
    use std::str::FromStr;

    let matches = App::new("Midas")
        .version(crate_version!())
        .author("Will Cooper")
        .about("Distributed network based parallel computing system")
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .takes_value(true)
            .help("Specifies either Host (server) or participant (client) mode.")
            .possible_values(&["host", "participant"])
            .required(true))
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
        .get_matches();

    let ip_address = matches.value_of("socket address").unwrap();

    match matches.value_of("mode").unwrap() {
        "host" => {
            let mut host = host::Host::new(ip_address);
            loop {
                host.check_events();
            }
        },
        "participant" => {

            let constants = get_constants();
            let instructions = get_instructions();

            let mut participant = participant::Participant::new(ip_address);


            loop {
                participant.check_events(&constants, &instructions);
            }

        },
        _ => unreachable!()
    };
}
