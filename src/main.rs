#![feature(available_concurrency)]
#[macro_use]
extern crate capture;

mod host;
mod participant;
mod messages;
mod lua;

extern crate clap;

use clap::{crate_version, Arg, App, SubCommand};
use std::thread;

use crossterm::event::{read, Event, poll};
use std::time::Duration;

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

    match app_matches.subcommand() {
        ("host", host_matches) => {
            let mut host = host::Host::new(ip_address.as_str());

            let script_path = host_matches.unwrap().value_of("Lua script").unwrap();

            loop {
                if let Ok(true) = poll(Duration::from_secs(0)) {
                    match read().unwrap() {
                        Event::Key(key_event) => {
                            match key_event.code {
                                crossterm::event::KeyCode::Char('e') => {
                                    host.start_participants(script_path);
                                },
                                crossterm::event::KeyCode::Char('d') => {
                                    host.display_participants();
                                },
                                crossterm::event::KeyCode::Char('c') => {
                                    host.display_participant_count();
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                host.check_events();
            }
        },
        ("participant", participant_matches) => {



            //Integer containing the number of extra threads to spawn.
            //This can be written as spawn_thread_count = total_thread_count - 1
            //Since one of the total threads is the main thread, which is not spawned
            let spawn_thread_count: usize = if participant_matches.unwrap().is_present("thread count")
            {
                match participant_matches.unwrap().value_of("thread count") {
                    Some(thread_count) => {
                        thread_count.parse::<usize>().unwrap() - 1
                    },
                    None => {
                        thread::available_concurrency().unwrap().get()
                    }
                }
            }
            else {
                0
            };

            let participant_name = String::from(participant_matches.unwrap().value_of("participant name").unwrap());

            for i in 0..spawn_thread_count {

                thread::Builder::new().name(
                    format!("thread_{}-{}", &participant_name, i)
                ).spawn(capture!(clone participant_name, clone ip_address, clone i in move || {
                    let mut participant = participant::Participant::new(format!("{}-{}", participant_name, i), ip_address.as_str());

                    loop {
                        participant.check_events();
                    }
                })).unwrap();
            }


            //This final participant is executed in the main thread
            let mut participant = participant::Participant::new(
                if spawn_thread_count == 0 {
                        format!("{}", participant_name)
                    }
                    else {
                        format!("{}-{}", participant_name, spawn_thread_count)
                    }
                ,

                ip_address.as_str());

            loop {
                participant.check_events();
            }

        },
        _ => unreachable!()
    };
}
