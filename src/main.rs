mod operand;
mod instructions;
mod host;
mod participant;
mod messages;

extern crate clap;

use clap::{Arg, App};

fn main() {

    let matches = App::new("Midas")
        .version("0.1.3")
        .author("Will Cooper")
        .about("Distributed network based paralell computing system")
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .takes_value(true)
            .help("Either host, participant or compile")
            .possible_values(&["host", "participant", "compile"])
            .required(true))
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .takes_value(true)
            .help("IP address to host/connect to")
            .validator( |_value| Ok(()))
            .required(true))
        .get_matches();

    let ip_address = matches.value_of("address").unwrap();

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
        "compile" => println!("Compile mode"),
        _ => unreachable!()
    };
}
