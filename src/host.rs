use crate::messages::{Message, UiEvents, ParticipantStatus, Severity, NodeType};

use message_io::network::Endpoint;

use bimap::BiMap;

use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, AnyLuaValue, LuaTable, LuaFunctionCallError};
use std::io::Read;

use crate::lua::SerdeLuaTable;

use crate::messages::HostEvent;
use crossbeam_channel::{Receiver, Sender};


pub struct Host<'a> {
    participants: BiMap<String, Endpoint>,
    //event_queue: EventQueue<HostEvent>,
    network: Network,

    //ui_sender: Option<EventSender<UiEvents>>,

    command_receiver: Receiver<HostEvent>,
    message_sender: Sender<UiEvents>,

    participants_finished: usize,
    participants_startedwith: BiMap<String, Endpoint>,


    lua: Lua<'a>
}

impl<'a> Host<'a> {

    pub fn new(command_receiver: Receiver<HostEvent>,
               command_sender: Sender<HostEvent>,
               message_sender: Sender<UiEvents>,
               server_address: &str) -> Result<Self, String> {

        let network_sender = command_sender.clone();

        let mut network = Network::new(move |net_event| network_sender.send(HostEvent::Network(net_event)).unwrap());

        let mut lua = Lua::new();

        lua.openlibs();

        match network.listen(Transport::Tcp, server_address) {
            Ok(_) => message_sender.send(UiEvents::Log(NodeType::Host, format!("Host running at {}", server_address), Severity::Info)).unwrap(),
            Err(e) => return Err(format!("Can not listen at {} - {}", server_address, e))
        };

        Ok(Host {
            participants: BiMap::new(),
            command_receiver,
            network,
            participants_finished: 0,
            participants_startedwith: BiMap::new(),
            message_sender,
            lua
        })
    }

    fn send_data(& mut self) -> Result<(), String> {
        //Extract the 'generate data' function from the Lua script.
        let generate_data_option: Option<hlua::LuaFunction<_>> = self.lua.get("generate_data");

        match generate_data_option {
            Some(mut generate_data) => {
                let endpoint_count = self.participants.len();


                //Call generate_data function for each endpoint, and send the resultant data
                for (i, (_name, endpoint)) in self.participants.iter().enumerate() {
                    let result_option: Result<LuaTable<_>, _> = generate_data.call_with_args((i as i32, endpoint_count as i32));

                    match result_option {
                        Ok(mut result) => {
                            let list: SerdeLuaTable = result.iter::<AnyLuaValue, AnyLuaValue>().map(|pair| pair.unwrap()).collect();


                            self.network.send(*endpoint, Message::VectorHTP(list));
                        }
                        Err(e) => {


                            return match e {
                                LuaFunctionCallError::LuaError(e) => {
                                    Err(format!("Error in `generate_data` function - {}", e))
                                }
                                LuaFunctionCallError::PushError(e) => {
                                    Err(format!("Error in `generate_data` function - PushError: {:?}", e))
                                }
                            }

                        }
                    }


                }

                Ok(())
            }
            None => {
                Err(format!("`generate_data` function does not exist in script."))
            }
        }


    }

    fn send_code(& mut self, code: String) {
        for (_name, endpoint) in self.participants.iter() {
            self.network.send(*endpoint, Message::Code(code.clone()));
        }
    }

    fn execute(& mut self) {
        for (_, endpoint) in self.participants.iter() {
            self.network.send(*endpoint, Message::Execute);
        }
    }

    pub fn start_participants(& mut self, path: &str) {


        use std::fs::File;

        match File::open(path) {
            Ok(mut fh) => {


                let mut source_code = String::new();

                match fh.read_to_string(&mut source_code) {
                    Ok(_) => {
                        let message_sender = self.message_sender.clone();

                        self.lua.set("_print", hlua::function1(move |message: String| {
                            message_sender.send(UiEvents::Log(NodeType::Host, message, Severity::Stdout)).unwrap();
                        }));

                        match self.lua.execute::<()>(source_code.as_str()) {
                            Ok(_) => {

                                self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Starting calculations on {} participants.", self.participants.len()), Severity::Starting)).unwrap();

                                self.participants_finished = 0;

                                self.participants_startedwith = self.participants.clone();

                                match self.send_data() {
                                    Ok(_) => {


                                        self.send_code(source_code);

                                        self.execute();
                                    }
                                    Err(e) => {
                                        self.message_sender.send(UiEvents::Log(NodeType::Host, e, Severity::Error)).unwrap();

                                    }
                                }
                            }
                            Err(e) => {
                                self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Bad Lua script - {}", e), Severity::Error)).unwrap();

                            }
                        }
                    }
                    Err(e) => {
                        self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Error parsing script - {}", e), Severity::Error)).unwrap();

                    }
                }




            }
            Err(e) => {
                self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Error opening script - {}", e), Severity::Error)).unwrap();

            }
        }




    }

    pub fn check_events(& mut self) {

        match self.command_receiver.recv(/*Duration::from_micros(0)*/) {
            Ok(event) => match event {
                HostEvent::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {


                        match message {
                            Message::Register(name) => {
                                if self.participants.contains_left(&name) {
                                    self.message_sender.send(UiEvents::Log(NodeType::Participant(name.clone()), format!("Could not register participant due to name conflict"), Severity::Warning)).unwrap();
                                    self.network.remove_resource(endpoint.resource_id());
                                }
                                else {
                                    self.participants.insert(name.clone(), endpoint);
                                    self.message_sender.send(UiEvents::ParticipantRegistered(endpoint, name.clone())).unwrap();
                                    //self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Idle, endpoint, name)).unwrap();


                                }
                            },
                            Message::Unregister => {
                                {
                                    let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                    self.message_sender.send(UiEvents::ParticipantUnregistered(endpoint_name.clone())).unwrap();
                                }
                                self.participants.remove_by_right(&endpoint);
                            },
                            Message::VectorPTH(data) => {

                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();

                                self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Idle, endpoint, endpoint_name.clone())).unwrap();

                                if self.participants_startedwith != self.participants {
                                    self.message_sender.send(UiEvents::Log(NodeType::Participant(endpoint_name.clone()), format!("Some participants have disconnected/connected before execution could complete."), Severity::Error)).unwrap();

                                }
                                else {
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

                                        let interpret_results_option: Option<hlua::LuaFunction<_>> = self.lua.get("interpret_results");

                                        match interpret_results_option {
                                            Some(mut interpret_results) => {
                                                // Get return value
                                                match interpret_results.call::<String>() {
                                                    Ok(return_code) => {

                                                        self.message_sender.send(UiEvents::InterpretResultsReturn(return_code)).unwrap();
                                                    }
                                                    Err(e) => {
                                                        self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Error in `interpret_results` function - {}", e), Severity::Error)).unwrap();

                                                    }
                                                }

                                            }
                                            None => {

                                                self.message_sender.send(UiEvents::Log(NodeType::Host, format!("`interpret_results` function does not exist in script."), Severity::Error)).unwrap();
                                            }
                                        }




                                    }
                                }
                            },
                            Message::ParticipantError(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::Log(NodeType::Participant(endpoint_name.clone()), err, Severity::Error)).unwrap();
                            },
                            Message::ParticipantWarning(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::Log(NodeType::Participant(endpoint_name.clone()), err, Severity::Warning)).unwrap();
                            },
                            Message::Whisper(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::Log(NodeType::Participant(endpoint_name.clone()), err, Severity::Info)).unwrap();
                            },
                            Message::Progress(progress) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::ParticipantProgress(endpoint_name.clone(),progress)).unwrap();

                            },
                            Message::Paused => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Paused, endpoint, endpoint_name.clone())).unwrap();

                            },
                            Message::Executing => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Calculating, endpoint, endpoint_name.clone())).unwrap();

                            },
                            Message::Stdout(output) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::Log(NodeType::Participant(endpoint_name.clone()), output, Severity::Stdout)).unwrap();

                            }
                            _ => {

                                panic!("Invalid message received by host ({:?})", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(_endpoint) => {
                        //Participant has connected to the host, but at this stage has not yet registered
                        //self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Participant added: {}", endpoint), Severity::Info)).unwrap();

                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Participant disconnected without unregistering
                        match self.participants.get_by_right(&endpoint)
                        {
                            Some(endpoint_name) => {

                                self.message_sender.send(UiEvents::ParticipantUnregistered(endpoint_name.clone())).unwrap();

                                self.participants.remove_by_right(&endpoint);
                            }
                            None => {

                            }
                        }


                    }
                    NetEvent::DeserializationError(_) => (),
                },
                HostEvent::Pause(endpoint) => {
                    self.network.send(endpoint, Message::Pause);

                },
                HostEvent::Play(endpoint) => {
                    self.network.send(endpoint, Message::Play);

                },
                HostEvent::Kill(endpoint) => {
                    self.network.send(endpoint, Message::Kill);
                },
                HostEvent::Begin(path) => {
                    self.start_participants(path.as_str());
                },

                HostEvent::PlayAll => {
                    for (_, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Play);

                    }
                },

                HostEvent::PauseAll => {
                    for (_, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Pause);

                    }
                },

                HostEvent::KillAll => {
                    for (_, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Kill);
                    }
                },

                HostEvent::RemoveAll => {
                    for (_, endpoint) in self.participants.iter() {
                        self.network.remove_resource(endpoint.resource_id());
                    }
                },
            },
            Err(e) => {
                eprintln!("Receive error in Host::check_events() - {}", e);
            }
        }


    }

}
