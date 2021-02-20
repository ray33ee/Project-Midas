mod host;
mod participant;
mod messages;
mod lua;


extern crate clap;


use clap::{crate_version, Arg, App};

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
        .arg(Arg::with_name("Lua script")
            .short("s")
            .long("script")
            .takes_value(true)
            .help("Lua script to run")
            .validator(|value| Ok(()))
            .required(true))
        .get_matches();

    let ip_address = matches.value_of("socket address").unwrap();

    match matches.value_of("mode").unwrap() {
        "host" => {
            let mut host = host::Host::new(ip_address);

            host.add_code(matches.value_of("Lua script").unwrap());



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
