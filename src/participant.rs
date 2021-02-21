use crate::messages::Message;

use message_io::network::Endpoint;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, LuaTable, AnyLuaValue};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Participant<'a> {
    host_endpoint: Endpoint,
    event_queue: EventQueue<Event>,
    network: Network,

    lua: Lua<'a>,
    paused: bool,
}

impl<'a> Participant<'a> {

    pub fn new(name: String, server_address: &str) -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let mut lua = Lua::new();

        lua.openlibs();

        if let Ok(host_endpoint) = network.connect(Transport::Tcp, server_address) {
            println!("Participant '{}' connected to host ({})", name, server_address);

            network.send(host_endpoint, Message::Register(name.clone()));

            Participant {
                host_endpoint,
                event_queue,
                network,
                lua,
                paused: false
            }
        }
        else {
            panic!("Can not connect to the server by TCP to {}", server_address);

        }


    }

    pub fn check_events(& mut self) {


        match self.event_queue.receive() {
            Event::Network(net_event) => match net_event {
                NetEvent::Message(_, message) => {
                    match message {
                        Message::Code(code) => {

                            match self.lua.execute::<()>(code.as_str()) {
                                Ok(_) => {}
                                Err(e) => {
                                    self.network.send(self.host_endpoint, Message::ParticipantError(String::from(format!("LuaError on receive Message::Code - {:?}", e))));
                                    panic!("LuaError on receive Message::Code - {:?}", e);
                                }
                            }

                        },
                        Message::VectorHTP(data) => {

                            let mut arr = self.lua.empty_array("global_data");


                            for (_, value) in data.iter().enumerate() {
                                arr.set(value.0.clone(), value.1.clone());
                            }

                        },
                        Message::Pause => {
                            self.paused = true;
                        },
                        Message::Play => {
                            self.paused = false;
                        },
                        Message::Stop => {

                            self.paused = false;
                        },
                        Message::Execute => {

                            match self.lua.get::<hlua::LuaFunction<_>, _>("execute_code")
                            {
                                Some (mut generate_data) => {
                                    match generate_data.call::<LuaTable<_>>() {
                                        Ok(mut result) => {
                                            let list: crate::lua::SerdeLuaTable = result.iter::<AnyLuaValue, AnyLuaValue>().map(|pair| pair.unwrap()).collect();

                                            self.network.send(self.host_endpoint, Message::VectorPTH(list));
                                        }
                                        Err(e) => {
                                            self.network.send(self.host_endpoint, Message::ParticipantError(String::from(format!("LuaError on receive Message::Execute - {:?}", e))));
                                            panic!("LuaError on receive Message::Execute - {:?}", e);
                                        }
                                    };
                                },
                                None => {
                                    self.network.send(self.host_endpoint, Message::ParticipantError(String::from("LuaError on receive Message::Execute - Function 'execute_code' does not exist.")));
                                    panic!("LuaError on receive Message::Execute - Function 'execute_code' does not exist.");
                                }
                            }


                            //Call generate_data function for each endpoint, and send the resultant data





                            }
                        _ => { /*panic!("Invalid message {:?}", message);*/ }
                    }
                }
                NetEvent::AddedEndpoint(_endpoint) => {},
                NetEvent::RemovedEndpoint(_endpoint) => {
                    println!("Server Disconnected");
                }
                NetEvent::DeserializationError(_) => (),
            }
        }


    }

}
