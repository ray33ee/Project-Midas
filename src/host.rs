use crate::messages::Message;

use message_io::network::Endpoint;

use bimap::BiMap;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, AnyLuaValue, LuaTable};
use std::io::Read;

use crate::lua::SerdeLuaTable;
use std::time::Duration;

enum Event {
    Network(NetEvent<Message>),
    SendCode(String),
    SendData,
    Pause(Endpoint),
    Play(Endpoint),
    Stop(Endpoint),
    Execute
}

pub struct Host<'a> {
    participants: BiMap<String, Endpoint>,
    event_queue: EventQueue<Event>,
    network: Network,

    participants_finished: usize,
    participants_startedwith: BiMap<String, Endpoint>,

    lua: Lua<'a>
}

impl<'a> Host<'a> {

    pub fn new(server_address: &str) -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let mut lua = Lua::new();

        lua.openlibs();

        match network.listen(Transport::Tcp, server_address) {
            Ok(_) => println!("TCP Server running at {}", server_address),
            Err(e) => panic!("Can not listen at {} - {}", server_address, e)
        }

        Host {
            participants: BiMap::new(),
            event_queue,
            network,
            participants_finished: 0,
            participants_startedwith: BiMap::new(),
            lua
        }
    }

    /*pub fn add_code(& mut self, path: & str) {
        // Get source code

        use std::fs::File;

        let mut fh = File::open(path).unwrap();

        let mut source_code = String::new();

        fh.read_to_string(& mut source_code).unwrap();

        match self.lua.execute::<()>(source_code.as_str()) {
            Ok(_) => {}
            Err(e) => { panic!("LuaError: {:?}", e); }
        }


    }*/

    pub fn start_participants(& mut self, path: &str) {

        use std::fs::File;

        let mut fh = File::open(path).unwrap();

        let mut source_code = String::new();

        fh.read_to_string(& mut source_code).unwrap();

        match self.lua.execute::<()>(source_code.as_str()) {
            Ok(_) => {}
            Err(e) => { panic!("LuaError: {:?}", e); }
        }

        self.participants_startedwith = self.participants.clone();

        self.event_queue.sender().send(Event::SendData);

        self.event_queue.sender().send(Event::SendCode(source_code));

        self.event_queue.sender().send(Event::Execute);
    }

    pub fn display_participants(& self) {
        println!("Participants: {:?}", self.participants);
    }

    pub fn display_participant_count(& self) {
        println!("Participant count: {}", self.participants.len());
    }

    pub fn check_events(& mut self) {

        match self.event_queue.receive_timeout(Duration::from_micros(1)) {
            Some(event) => match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {


                        match message {
                            Message::Register(name) => {
                                println!("Register participant '{}'", name);
                                if self.participants.contains_left(&name) {
                                    println!("Participant {} could not be registered. Participant with this name already exists.", name);
                                    self.network.remove_resource(endpoint.resource_id());
                                }
                                else {
                                    self.participants.insert(name, endpoint);
                                }
                            },
                            Message::Unregister => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                println!("Unregister participant '{}'", endpoint_name);
                                self.participants.remove_by_right(&endpoint);
                            },
                            Message::VectorPTH(data) => {

                                if self.participants_startedwith != self.participants {
                                    println!("Some participants have disconnected/connected before execution could complete.")
                                }
                                else {
                                    let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();

                                    println!("Received data from '{}'.", endpoint_name);

                                    //A participant has finished, so increment the count
                                    self.participants_finished += 1;

                                    //If this is the first participant, initialise the results variable
                                    if self.participants_finished == 1 {
                                        self.lua.empty_array("results");
                                    }

                                    {
                                        //Create temporary global array called 'tmp_table'
                                        let mut arr = self.lua.empty_array("tmp_table");

                                        // Copy data to temporary array
                                        for (_, value) in data.iter().enumerate() {
                                            arr.set(value.0.clone(), value.1.clone());
                                        }
                                    }

                                    //Move the temporary table to to the global results
                                    self.lua.execute::<()>(format!("results[{}] = tmp_table", self.participants_finished).as_str()).unwrap();

                                    /*println!("    len:      {}", self.participants.len());
                                    println!("    finished: {}", self.participants_finished);
                                    println!("    parts_:   {:?}", self.participants);
                                    println!("    partsS:   {:?}", self.participants_startedwith);*/

                                    // Test to see if all participants have finished
                                    if self.participants.len() == self.participants_finished {


                                        // Execute 'interpret_results' function
                                        let mut interpret_results: hlua::LuaFunction<_> = self.lua.get("interpret_results").unwrap();

                                        // Get return value
                                        let return_code: String = interpret_results.call().unwrap();

                                        println!("All participants finished. interpret_results return code: {}", return_code);


                                        self.participants_finished = 0;
                                    }
                                }
                            },
                            Message::ParticipantError(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                println!("Participant '{}' Error: {}", endpoint_name, err);
                            },
                            Message::ParticipantWarning(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                println!("Participant '{}' Warning: {}", endpoint_name, err);
                            }
                            _ => {
                                panic!("Invalid message received by host ({:?})", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(endpoint) => {
                        println!("Client Added {}", endpoint);
                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Client disconnected without unregistering
                        let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                        println!("Client '{}' has disconnected", endpoint_name);

                        self.participants.remove_by_right(&endpoint);
                    }
                    NetEvent::DeserializationError(_) => (),
                },
                Event::SendCode(code) => {
                    println!("Sending code to all {} participant(s)", self.participants.len());
                    for (_name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Code(code.clone()));
                    }
                },
                Event::SendData => {
                    println!("Sending data to all {} participant(s)", self.participants.len());

                    //Extract the 'generate data' function from the Lua script.
                    let mut generate_data: hlua::LuaFunction<_> = self.lua.get("generate_data").unwrap();

                    let endpoint_count = self.participants.len();


                    //Call generate_data function for each endpoint, and send the resultant data
                    for (i, (_name, endpoint)) in self.participants.iter().enumerate() {
                        let mut result: LuaTable<_> = generate_data.call_with_args((i as i32, endpoint_count as i32)).unwrap();

                        let list: SerdeLuaTable = result.iter::<AnyLuaValue, AnyLuaValue>().map(|pair| pair.unwrap()).collect();


                        self.network.send(*endpoint, Message::VectorHTP(list));
                    }
                },
                Event::Pause(endpoint) => {
                    self.network.send(endpoint, Message::Pause);
                },
                Event::Play(endpoint) => {
                    self.network.send(endpoint, Message::Play);
                },
                Event::Stop(endpoint) => {
                    self.network.send(endpoint, Message::Stop);
                },
                Event::Execute => {
                    println!("Sending execute command to all {} participant(s)", self.participants.len());
                    for (_name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Execute);
                    }
                },
            },
            None => {

            }
        }

    }

}
