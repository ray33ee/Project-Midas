#![feature(available_concurrency)]
#[macro_use]
extern crate capture;

mod host;
mod participant;
mod messages;
mod lua;
mod ui;

extern crate clap;

use clap::{crate_version, Arg, App, SubCommand};
use std::thread;

use crate::ui::Panel;
use crate::host::Host;

use crate::messages::{HostEvent, UiEvents};
use crossbeam_channel::unbounded;

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
                .min_values(0)
                .takes_value(true)
                .required(false))
            .about("Executes Midas as a participant"))
        .get_matches();

    let ip_address = String::from(app_matches.value_of("socket address").unwrap());

    //Setup the channels of communication between Host code and ui
    let (command_sender, command_receiver) = unbounded::<HostEvent>();
    let (message_sender, message_receiver) = unbounded::<UiEvents>();

    match app_matches.subcommand() {
        ("host", host_matches) => {
            match Host::new(command_receiver, command_sender.clone(), message_sender,ip_address.as_str()) {
                Ok(mut host) => {
                    let script_path = host_matches.unwrap().value_of("Lua script").unwrap();

                    let mut panel = Panel::new(command_sender.clone(), message_receiver, String::from(script_path));

                    thread::spawn(move ||
                        loop {
                            host.check_events()
                        }
                    );

                    loop {
                        panel.tick();
                    }
                },
                Err(error) => {
                    println!("Host Error - {}", error);
                }
            }


        },
        ("participant", participant_matches) => {


            let thread_count: usize = if participant_matches.unwrap().is_present("thread count")
            {
                match participant_matches.unwrap().value_of("thread count") {
                    Some(thread_count) => {
                        thread_count.parse::<usize>().unwrap()
                    },
                    None => {
                        thread::available_concurrency().unwrap().get()
                    }
                }
            }
            else {
                1
            };

            let participant_name = String::from(participant_matches.unwrap().value_of("participant name").unwrap());

            let mut thread_vec = Vec::new();

            for i in 0..thread_count {

                thread_vec.push(thread::Builder::new()
                    .name(format!("thread_{}-{}", &participant_name, i))
                    .spawn(capture!(clone participant_name, clone ip_address, clone i, clone thread_count in move || {
                        let mut participant = participant::Participant::new(
                            if thread_count == 1 {
                                format!("{}", participant_name)
                            }
                            else {
                                format!("{}-{}", participant_name, i)
                            },ip_address.as_str());

                        while let Ok(_) = participant.tick() {

                        }

                })).unwrap());
            }

            //Under normal operation, each spawned thread and main thread should
            //continue execution indefinitely. Here we block the main thread to allow
            //the spawned threads to continue. If all the spawned threads end, then the
            //main thread will end, along with the application
            for thread_handle in thread_vec {
                thread_handle.join().unwrap();
            }

        },
        _ => unreachable!()
    };
}
