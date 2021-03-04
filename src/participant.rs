use hlua::{AnyLuaValue, Lua, LuaTable};
use message_io::network::{NetEvent, Network, Transport};

use crate::messages::Message;

use crossbeam_channel::{Sender, Receiver, unbounded, RecvTimeoutError};

use std::thread;
use std::time::Duration;

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

    fn recv_message(rec: & Receiver<NetEvent<Message>>, dur: Option<u64>) -> Option<Message> {
        let net_event = if let Some(duration) = dur {
            match rec.recv_timeout(Duration::from_micros(duration)) {
                Ok(msg) => {
                    msg
                }
                Err(receive_error) => match receive_error {
                    RecvTimeoutError::Disconnected => {
                        panic!("Participant::message_receiver has disconnected")
                    },
                    RecvTimeoutError::Timeout => {
                        return None;
                    }
                }
            }
        } else {
            rec.recv().expect("Participant::message_receiver has disconnected")
        };

        match net_event {
            NetEvent::Message(_, message) => {
                Some(message)
            },
            NetEvent::RemovedEndpoint(_endpoint) => {
                panic!("Server Disconnected. See Host for more details.");
            }
            _ => {
                None
            }
        }
    }

    pub fn tick(& mut self) -> Result<(), ()> {


        match self.message_receiver.recv() {
            Ok(nevent) => match nevent {
                    NetEvent::Message(_, message) => {
                        match message {
                            Message::Code(code) => {



                                let net_sender = self.network.clone();

                                self.lua.set("_print", hlua::function1(move |message: String| {
                                    net_sender.send(Message::Stdout(message));
                                }));

                                //Register the _check function which allows Lua script users to check the
                                //network and respond to pause/play and stop commands
                                let receiver = self.message_receiver.clone();
                                let net_sender = self.network.clone();



                                self.lua.set("_check", hlua::function0(move ||
                                    {
                                        let refy = & receiver;


                                        //println!("Check start");
                                        match Self::recv_message(refy, Some(0)) {
                                            Some(msg) => match msg {
                                                Message::Stop => {
                                                    panic!("This is a cheaty way to kill the thread, but fuck it, we'll do it live!");
                                                }
                                                Message::Pause => {

                                                    net_sender.send(Message::Paused).unwrap();
                                                    loop {
                                                        match Self::recv_message(refy, None) {
                                                            Some(ms) => match ms {
                                                                Message::Stop => {
                                                                    panic!("This is a cheaty way to kill the thread, but fuck it, we'll do it live!");
                                                                }
                                                                Message::Play => {

                                                                    net_sender.send(Message::Executing).unwrap();
                                                                    break;
                                                                }
                                                                _ => {
                                                                    println!("Inner: {:?}", msg);
                                                                }
                                                            }
                                                            None => {

                                                            }
                                                        }
                                                    }

                                                }
                                                _ => {
                                                    println!("MESSAGE RECEIVED DURING PAUSE Check: {:?}", msg);
                                                }
                                            }
                                            None => {

                                            }
                                        }
                                        //println!("Check finish");
                                    }
                                ));

                                //Register the _progress function which allows Lua script users to send
                                //data back to the host indicating how much progress the script has made
                                let net_sender = self.network.clone();

                                let mut last_progress_update = std::time::Instant::now();

                                self.lua.set("_progress", hlua::function2(move |prog: f32, delay: u32|
                                {
                                    if std::time::Instant::now().duration_since(last_progress_update).as_millis() > delay as u128 {
                                        net_sender.send(Message::Progress(prog)).unwrap();
                                        last_progress_update = std::time::Instant::now();
                                    }

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
                            Message::Stop => {
                                panic!("This is a cheaty way to kill the thread, but fuck it, we'll do it live!");
                            },
                            Message::Execute => {

                                match self.lua.get::<hlua::LuaFunction<_>, _>("execute_code")
                                {
                                    Some(mut generate_data) => {

                                        self.network.send(Message::Executing).unwrap();

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
                            },

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
