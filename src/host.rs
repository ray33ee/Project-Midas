use crate::messages::{Message, UiEvents, ParticipantStatus, Severity, NodeType};

use message_io::network::Endpoint;

use bimap::BiMap;

use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, AnyLuaValue, LuaTable};
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
    command_sender: Sender<HostEvent>,
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
            Ok(_) => println!("TCP Server running at {}", server_address),
            Err(e) => return Err(format!("Can not listen at {} - {}", server_address, e))
        }


        Ok(Host {
            participants: BiMap::new(),
            command_receiver,
            command_sender: command_sender.clone(),
            network,
            participants_finished: 0,
            participants_startedwith: BiMap::new(),
            message_sender,
            lua
        })
    }

    pub fn start_participants(& mut self, path: &str) {

        self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Starting calculations."), Severity::Whisper)).unwrap();

        use std::fs::File;

        let mut fh = File::open(path).unwrap();

        let mut source_code = String::new();

        fh.read_to_string(& mut source_code).unwrap();

        match self.lua.execute::<()>(source_code.as_str()) {
            Ok(_) => {}
            Err(e) => { panic!("DEPRECIATED LuaError: {:?}", e); }
        }

        self.participants_finished = 0;

        self.participants_startedwith = self.participants.clone();

        self.command_sender.send(HostEvent::SendData).unwrap();

        self.command_sender.send(HostEvent::SendCode(source_code)).unwrap();

        self.command_sender.send(HostEvent::Execute).unwrap();
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
                                    self.message_sender.send(UiEvents::ParticipantUnregistered(endpoint, endpoint_name.clone())).unwrap();
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


                                        // Execute 'interpret_results' function
                                        let mut interpret_results: hlua::LuaFunction<_> = self.lua.get("interpret_results").unwrap();

                                        // Get return value
                                        let return_code: String = interpret_results.call().unwrap();

                                        self.message_sender.send(UiEvents::InterpretResultsReturn(return_code)).unwrap();

                                    }
                                }
                            },
                            Message::ParticipantError(err) => {
                                //let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                //self.message_sender.send(UiEvents::ParticipantLog(endpoint, err, endpoint_name.clone(), Severity::Error)).unwrap();
                            },
                            Message::ParticipantWarning(err) => {
                               // let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                //self.message_sender.send(UiEvents::ParticipantLog(endpoint, err, endpoint_name.clone(), Severity::Warning)).unwrap();
                            },
                            Message::Whisper(err) => {
                               // let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                //self.message_sender.send(UiEvents::ParticipantLog(endpoint, err, endpoint_name.clone(), Severity::Whisper)).unwrap();
                            },
                            Message::Progress(progress) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.message_sender.send(UiEvents::ParticipantProgress(endpoint, endpoint_name.clone(),progress)).unwrap();

                            }
                            _ => {

                                panic!("Invalid message received by host ({:?})", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(endpoint) => {
                        //Client has connected to the host, but at this stage has not yet registered
                        self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Client added: {}", endpoint), Severity::Whisper)).unwrap();

                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Client disconnected without unregistering
                        {
                            let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                            self.message_sender.send(UiEvents::ParticipantUnregistered(endpoint, endpoint_name.clone())).unwrap();
                        }
                        self.participants.remove_by_right(&endpoint);
                    }
                    NetEvent::DeserializationError(_) => (),
                },
                HostEvent::SendCode(code) => {
                    self.message_sender.send(UiEvents::Log(NodeType::Host, format!("Executing on {} participant(s)", self.participants.len()), Severity::Whisper)).unwrap();

                    for (_name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Code(code.clone()));
                    }
                },
                HostEvent::SendData => {

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
                HostEvent::Pause(endpoint) => {
                    self.network.send(endpoint, Message::Pause);
                    let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                    self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Paused, endpoint, endpoint_name.clone())).unwrap();

                },
                HostEvent::Play(endpoint) => {
                    self.network.send(endpoint, Message::Play);
                    let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                    self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Calculating, endpoint, endpoint_name.clone())).unwrap();

                },
                HostEvent::Kill(endpoint) => {
                    self.network.send(endpoint, Message::Stop);
                },
                HostEvent::Execute => {
                    for (name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Execute);
                        self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Calculating, *endpoint, name.clone())).unwrap();
                    }
                },
                HostEvent::Begin(path) => {
                    self.start_participants(path.as_str());
                },

                HostEvent::PlayAll => {
                    for (name, endpoint) in self.participants.iter() {
                        self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Calculating, *endpoint, name.clone())).unwrap();
                        self.network.send(*endpoint, Message::Play);

                    }
                },

                HostEvent::PauseAll => {
                    for (name, endpoint) in self.participants.iter() {
                        self.message_sender.send(UiEvents::ChangeStatusTo(ParticipantStatus::Paused, *endpoint, name.clone())).unwrap();
                        self.network.send(*endpoint, Message::Pause);

                    }
                },

                HostEvent::KillAll => {
                    for (_, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Stop);
                    }
                }
            },
            Err(_e) => {
                //println!("Receive error in host - {}", e);
            }
        }


    }

}
