use crate::messages::Message;

use message_io::network::Endpoint;

use std::collections::HashSet;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, AnyLuaValue, LuaTable};
use std::io::Read;

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

    data: (Vec<f64>, usize),

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
            data: (Vec::new(), 0),
            lua
        }
    }

    pub fn add_code(& mut self, path: & str) {
        // Get source code

        use std::fs::File;

        let mut fh = File::open(path).unwrap();

        let mut source_code = String::new();

        fh.read_to_string(& mut source_code);

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

            fh.read_to_string(& mut source_code);

            self.event_queue.sender().send(Event::SendCode(source_code));

            self.event_queue.sender().send(Event::Execute);
        }
    }

    pub fn check_events(& mut self) {




        match self.event_queue.receive() {
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
                        Message::VectorPTH(mut data) => {
                            //Save data to computer path using endpoint and time and date as file name

                            println!("Data received: {:?}", data);
                            self.data.0.append(& mut data);
                            self.data.1 += 1;

                            // Test to see if all participants have finished
                            if self.participants.len() == self.data.1 {
                                {
                                    //Create global array called 'results'
                                    let mut arr = self.lua.empty_array("results");

                                    // Copy data to results
                                    for (i, value) in self.data.0.iter().enumerate() {
                                        arr.set((i + 1) as i32, *value);
                                    }
                                }
                                // Execute 'interpret_results' function
                                let mut interpret_results: hlua::LuaFunction<_> = self.lua.get("interpret_results").unwrap();

                                println!("Here!");

                                // Get return value
                                let return_code: String = interpret_results.call().unwrap();

                                println!("Return code: {}", return_code);

                                self.data.0.clear();
                                self.data.1 = 0;
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
            Event::SendCode(  code) => {

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

                    let list: Vec<f64> = result.iter::<i32, f64>().map(|pair| pair.unwrap().1).collect();

                    println!("{}", list.len());

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
        }



    }

}
