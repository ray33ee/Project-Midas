use hlua::{AnyLuaValue, Lua, LuaTable};
use message_io::network::{NetEvent, Network, Transport};

use crate::messages::Message;

use crossbeam_channel::{Sender, Receiver, unbounded};

use std::thread;

pub struct Participant<'a> {

    network: Sender<Message>,

    //message_sender: Sender<NetEvent<Message>>,
    message_receiver: Receiver<NetEvent<Message>>,

    lua: Lua<'a>,
}

impl<'a> Participant<'a> {

    pub fn new(name: String, server_address: &str) -> Self {

        let (message_sender, message_receiver) = unbounded();


        let mut lua = Lua::new();

        lua.openlibs();

        let (net_sender, net_receiver) = unbounded();

        let network_sender = message_sender.clone();

        let mut network = Network::new(move |net_event| network_sender.send(net_event).unwrap());

        match network.connect(Transport::Tcp, server_address) {
            Ok(host_endpoint) => {
                println!("Participant '{}' connected to host ({})", name, server_address);

                // The following thread monitors the net_sender/net_receiver channel and sends any data
                // it receives accross the network. This allows us to have multiple senders to the network
                thread::spawn(move ||
                    {


                        loop {
                            match net_receiver.recv() {
                                Ok(message) => {
                                    network.send(host_endpoint, message);
                                }
                                Err(_) => {

                                }
                            }
                        }
                    }
                );

                // Register the participant
                net_sender.send(Message::Register(String::from(name))).unwrap();

                Participant {
                    network: net_sender,
                    message_receiver,
                    lua
                }
            }
            Err(e) => {
                panic!("Could not connect to {} - {}", server_address, e);
            }
        }


    }

    pub fn tick(& mut self) -> Result<(), ()> {


        match self.message_receiver.recv() {
            Ok(nevent) => match nevent {
                    NetEvent::Message(_, message) => {
                        match message {
                            Message::Code(code) => {

                                //Register the _check function which allows Lua script users to check the
                                //network and respond to pause/play and stop commands
                                let receiver = self.message_receiver.clone();
                                self.lua.set("_check", hlua::function0(move ||
                                    {
                                        println!("Check start");
                                        match receiver.recv() {
                                            Ok(netevent) => match netevent {
                                                NetEvent::Message(_, msg) =>  {
                                                    println!("Found from check: {:?}", msg);
                                                }
                                                _ => {}
                                            }
                                            Err(_) => {

                                            }
                                        }
                                        println!("Check finish");
                                    }
                                ));

                                //Register the _progress function which allows Lua script users to send
                                //data back to the host indicating how much progress the script has made
                                let net_sender = self.network.clone();
                                self.lua.set("_progress", hlua::function1(move |prog: f32|
                                {
                                    net_sender.send(Message::Progress(prog)).unwrap();
                                }));


                                match self.lua.execute::<()>(code.as_str()) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        self.network.send( Message::ParticipantError(String::from(format!("LuaError on receive Message::Code - {:?}", e)))).unwrap();
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

                                                self.network.send( Message::VectorPTH(list)).unwrap();
                                                //result
                                            }
                                            Err(e) => {
                                                self.network.send(Message::ParticipantError(String::from(format!("LuaError on receive Message::Execute (Lua function return type) - {:?}", e)))).unwrap();
                                                panic!("LuaError on receive Message::Execute - {:?}", e);
                                            }
                                        };



                                    },
                                    None => {
                                        self.network.send( Message::ParticipantError(String::from("LuaError on receive Message::Execute (Lua function call) - Function 'execute_code' does not exist."))).unwrap();
                                        panic!("LuaError on receive Message::Execute - Function 'execute_code' does not exist.");
                                    }
                                }
                            }
                            _ => {
                                self.network.send(Message::ParticipantError(format!("Invalid message {:?}", message))).unwrap();
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
            Err(_) => {

            }
        }

        Ok(())
    }

}
