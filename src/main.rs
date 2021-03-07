#![feature(available_concurrency)]

mod host;
mod participant;
mod messages;
mod lua;
mod ui;

extern crate clap;
extern crate serde;

use clap::{crate_version, Arg, App, SubCommand};
use std::thread;

use crate::ui::Panel;
use crate::host::Host;

use crate::messages::{HostEvent, UiEvents};
use crossbeam_channel::unbounded;
use message_io::network::{Network, Transport, NetEvent};

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
            .arg(Arg::with_name("participant name")
                .short("n")
                .long("name")
                .help("Name of the participant, used by the host to identify participants")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("thread count")
                .short("t")
                .long("threads")
                .help("Number of threads. If no number is supplied, the value is calculated automatically.")
                .validator(|value|
                    match value.parse::<i32>() {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Invalid number of threads: Could not convert {} to number because '{}'.", value, e))
                    }
                )
                .takes_value(true)
                .required(false))
            .about("Executes Midas as a participant"))
        .get_matches();

    let ip_address = app_matches.value_of("socket address").unwrap();

    //Setup the channels of communication between Host code and ui
    let (command_sender, command_receiver) = unbounded::<HostEvent>();
    let (message_sender, message_receiver) = unbounded::<UiEvents>();

    match app_matches.subcommand() {
        ("host", host_matches) => {
            match Host::new(command_receiver, command_sender.clone(), message_sender,ip_address) {
                Ok(mut host) => {
                    let script_path = host_matches.unwrap().value_of("Lua script").unwrap();

                    let mut panel = Panel::new(command_sender.clone(), message_receiver, script_path);

                    thread::spawn(move ||
                        loop {
                            host.check_events()
                        }
                    );

                    while let Ok(_) = panel.tick() {}
                },
                Err(error) => {
                    println!("Host Error - {}", error);
                }
            }


        },
        ("participant", participant_matches) => {


            let thread_count: usize = if participant_matches.unwrap().is_present("thread count")
            {
                let number = participant_matches.unwrap().value_of("thread count").unwrap();
                number.parse::<usize>().unwrap()
            }
            else {
                thread::available_concurrency().unwrap().get()
            };

            let participant_name = participant_matches.unwrap().value_of("participant name").unwrap();

            loop
            {
                println!("Searching for host...");

                {
                    let mut network = Network::new(move |_: NetEvent<()>| {});


                    while let Err(_) = network.connect(Transport::Tcp, ip_address) {

                    }

                }

                println!("Found host!");

                crossbeam::thread::scope(|s| {
                    let participant_name = participant_name;
                    let ip_address = ip_address;

                    for i in 0..thread_count {
                        s.builder()
                            .name(format!("thread_{}-{}", &participant_name, i))
                            .spawn(move |_| {
                                let mut participant = participant::Participant::new(
                                    if thread_count == 1 {
                                        format!("{}", participant_name)
                                    } else {
                                        format!("{}-{:03}", participant_name, i)
                                    }, ip_address).unwrap();

                                while let Ok(_) = participant.tick() {}
                            }).unwrap();
                    }
                }).unwrap();

                println!("Disconnected.");


            }

        },
        _ => unreachable!()
    };
}
