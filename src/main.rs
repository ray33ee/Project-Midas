mod operand;
mod instructions;
mod host;
mod participant;
mod messages;

extern crate clap;

use clap::{Arg, App};

fn main() {

    use std::net::SocketAddrV4;
    use std::str::FromStr;

    let matches = App::new("Midas")
        .version("0.1.4")
        .author("Will Cooper")
        .about("Distributed network based paralell computing system")
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .takes_value(true)
            .help("Specifies either Host (server) or participant (client) mode.")
            .possible_values(&["host", "participant"])
            .required(true))
        .arg(Arg::with_name("ip address")
            .short("a")
            .long("address")
            .takes_value(true)
            .help("IP address to host/connect to. Pleas specify Ip address and port, such as '192.168.0.1:4000'.")
            .validator( |value|
                    match SocketAddrV4::from_str(value.as_str()) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string())
                    }
                )
            .required(true))
        .get_matches();

    let ip_address = matches.value_of("ip address").unwrap();

    match matches.value_of("mode").unwrap() {
        "host" => {
            let mut host = host::Host::new(ip_address);
            loop {
                host.check_events();
            }
        },
        "participant" => {
            let mut participant = participant::Participant::new(ip_address);
            loop {
                participant.check_events();
            }
        },
        _ => unreachable!()
    };
}
