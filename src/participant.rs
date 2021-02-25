use hlua::{AnyLuaValue, Lua, LuaTable};
use message_io::network::{NetEvent, Network, Transport};
use message_io::network::Endpoint;

use crate::messages::Message;

use crossbeam_channel::{Receiver, unbounded};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Participant<'a> {
    host_endpoint: Endpoint,
    network: Network,

    //sender: Sender<Event>,
    receiver: Receiver<Event>,

    lua: Lua<'a>,
}

impl<'a> Participant<'a> {

    pub fn new(name: String, server_address: &str) -> Self {

        let (sender, receiver) = unbounded();

        let network_sender = sender.clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)).unwrap());

        let mut lua = Lua::new();

        lua.openlibs();

        match network.connect(Transport::Tcp, server_address) {
            Ok(host_endpoint) => {
                println!("Participant '{}' connected to host ({})", name, server_address);

                network.send(host_endpoint, Message::Register(name.clone()));

                Participant {
                    host_endpoint,
                    network,
                    //sender,
                    receiver,
                    lua
                }
            }
            Err(e) => {
                panic!("Could not connect to {} - {}", server_address, e);
            }
        }


    }

    pub fn tick(& mut self) -> Result<(), ()> {


        match self.receiver.recv() {
            Ok(event) => match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(_, message) => {
                        match message {
                            Message::Code(code) => {

                                let receiver = self.receiver.clone();

                                self.lua.set("check", hlua::function0(move ||
                                    {
                                        println!("Check start");
                                        match receiver.recv() {
                                            Ok(ev) => match ev {
                                                Event::Network(netevent) => match netevent {
                                                    NetEvent::Message(_, msg) =>  {
                                                        println!("Found from check: {:?}", msg);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            Err(_) => {

                                            }
                                        }
                                        println!("Check finish");
                                    }
                                ));



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
                            Message::Pause => {},
                            Message::Play => {},
                            Message::Stop => {},
                            Message::Execute => {
                                println!("Waiting first...");


                                println!("Now executing.");

                                match self.lua.get::<hlua::LuaFunction<_>, _>("execute_code")
                                {
                                    Some(mut generate_data) => {
                                        match generate_data.call::<LuaTable<_>>() {
                                            Ok(mut result) => {
                                                let list: crate::lua::SerdeLuaTable = result.iter::<AnyLuaValue, AnyLuaValue>().map(|pair| pair.unwrap()).collect();

                                                self.network.send(self.host_endpoint, Message::VectorPTH(list));
                                                //result
                                            }
                                            Err(e) => {
                                                self.network.send(self.host_endpoint, Message::ParticipantError(String::from(format!("LuaError on receive Message::Execute (Lua function return type) - {:?}", e))));
                                                panic!("LuaError on receive Message::Execute - {:?}", e);
                                            }
                                        };
                                    },
                                    None => {
                                        self.network.send(self.host_endpoint, Message::ParticipantError(String::from("LuaError on receive Message::Execute (Lua function call) - Function 'execute_code' does not exist.")));
                                        panic!("LuaError on receive Message::Execute - Function 'execute_code' does not exist.");
                                    }
                                }
                            }
                            _ => {
                                self.network.send(self.host_endpoint, Message::ParticipantError(format!("Invalid message {:?}", message)));
                                panic!("Invalid message {:?}", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(_endpoint) => {},
                    NetEvent::RemovedEndpoint(_endpoint) => {
                        println!("Server Disconnected. See Host for more details.");
                        return Err(())
                    }
                    NetEvent::DeserializationError(_) => (),
                }
            }
            Err(_) => {

            }
        }

        Ok(())
    }

}
