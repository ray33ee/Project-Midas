use crate::messages::Message;

use message_io::network::Endpoint;

use std::collections::HashSet;

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
    participants: HashSet<Endpoint>,
    event_queue: EventQueue<Event>,
    network: Network,

    participants_finished: usize,

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
            Err(_) => panic!("Can not listening at {}", server_address)
        }

        Host {
            participants: HashSet::new(),
            event_queue,
            network,
            participants_finished: 0,
            lua
        }
    }

    pub fn add_code(& mut self, path: & str) {
        // Get source code

        use std::fs::File;

        let mut fh = File::open(path).unwrap();

        let mut source_code = String::new();

        fh.read_to_string(& mut source_code).unwrap();

        match self.lua.execute::<()>(source_code.as_str()) {
            Ok(_) => {}
            Err(e) => { panic!("LuaError: {:?}", e); }
        }


    }

    pub fn test_participant_event(& mut self) {
        if self.participants.len() == 1 {
            self.event_queue.sender().send(Event::SendData);

            use std::fs::File;

            let mut fh = File::open(".\\docs\\sample_code.lua").unwrap();

            let mut source_code = String::new();

            fh.read_to_string(& mut source_code).unwrap();

            self.event_queue.sender().send(Event::SendCode(source_code));

            self.event_queue.sender().send(Event::Execute);
        }
    }

    pub fn check_events(& mut self) {

        match self.event_queue.receive_timeout(Duration::from_micros(1)) {
            Some(event) => match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {
                        println!("Message received");

                        match message {
                            Message::Register => {
                                println!("    Register participant");
                                self.participants.insert(endpoint);

                                self.test_participant_event();

                                println!("Set: {:?}", self.participants);
                            },
                            Message::Unregister => {
                                self.participants.remove(&endpoint);
                            },
                            Message::VectorPTH(data) => {

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

                                // Test to see if all participants have finished
                                if self.participants.len() == self.participants_finished {

                                    // Execute 'interpret_results' function
                                    let mut interpret_results: hlua::LuaFunction<_> = self.lua.get("interpret_results").unwrap();

                                    // Get return value
                                    let return_code: String = interpret_results.call().unwrap();

                                    println!("Return code: {}", return_code);

                                    self.participants_finished = 0;
                                }
                            },
                            _ => {
                                panic!("Invalid message received by host ({:?})", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(endpoint) => {
                        println!("Client Added {:?}", endpoint);
                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Client disconnected without unregistering
                        println!("Client Disconnected {:?}", endpoint);

                        self.participants.remove(&endpoint);
                        println!("Set: {:?}", self.participants);
                    }
                    NetEvent::DeserializationError(_) => (),
                },
                Event::SendCode(code) => {
                    for endpoint in self.participants.iter() {
                        self.network.send(*endpoint, Message::Code(code.clone()));
                    }
                },
                Event::SendData => {

                    //Extract the 'generate data' function from the Lua script.
                    let mut generate_data: hlua::LuaFunction<_> = self.lua.get("generate_data").unwrap();

                    let endpoint_count = self.participants.len();


                    //Call generate_data function for each endpoint, and send the resultant data
                    for (i, endpoint) in self.participants.iter().enumerate() {
                        let mut result: LuaTable<_> = generate_data.call_with_args((i as i32, endpoint_count as i32)).unwrap();

                        let list: SerdeLuaTable = result.iter::<AnyLuaValue, AnyLuaValue>().map(|pair| pair.unwrap()).collect();

                        println!("Generated table: {:?}", list);

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
                    for endpoint in self.participants.iter() {
                        self.network.send(*endpoint, Message::Execute);
                    }
                },
            },
            None => {

            }
        }

    }

}
