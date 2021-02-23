use crate::messages::{Message, UiEvents, ParticipantStatus};

use message_io::network::Endpoint;

use bimap::BiMap;

use message_io::events::{EventQueue, EventSender};
use message_io::network::{Network, NetEvent, Transport};

use hlua::{Lua, AnyLuaValue, LuaTable};
use std::io::Read;

use crate::lua::SerdeLuaTable;
use std::time::Duration;

use crate::messages::HostEvent;

pub struct Host<'a> {
    participants: BiMap<String, Endpoint>,
    event_queue: EventQueue<HostEvent>,
    network: Network,

    ui_sender: Option<EventSender<UiEvents>>,

    participants_finished: usize,
    participants_startedwith: BiMap<String, Endpoint>,

    lua: Lua<'a>
}

impl<'a> Host<'a> {

    pub fn new(server_address: &str) -> Result<Self, String> {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(HostEvent::Network(net_event)));

        let mut lua = Lua::new();

        lua.openlibs();

        match network.listen(Transport::Tcp, server_address) {
            Ok(_) => println!("TCP Server running at {}", server_address),
            Err(e) => return Err(format!("Can not listen at {} - {}", server_address, e))
        }


        Ok(Host {
            participants: BiMap::new(),
            event_queue,
            network,
            participants_finished: 0,
            participants_startedwith: BiMap::new(),
            ui_sender: None,
            lua
        })
    }

    pub fn get_host_sender(& mut self) -> EventSender<HostEvent> {
        self.event_queue.sender().clone()
    }

    pub fn set_ui_sender(& mut self, sender: EventSender<UiEvents>) {
        self.ui_sender = Some(sender);
    }

    pub fn start_participants(& mut self, path: &str) {

        self.ui_sender.as_ref().unwrap().send(UiEvents::HostMessage(format!("Starting calculations.")));

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

        self.event_queue.sender().send(HostEvent::SendData);

        self.event_queue.sender().send(HostEvent::SendCode(source_code));

        self.event_queue.sender().send(HostEvent::Execute);
    }

    pub fn display_participants(& self) {
        println!("DEPRECIATED Participants: {:?}", self.participants);
    }

    pub fn display_participant_count(& self) {
        println!("DEPRECIATED Participant count: {}", self.participants.len());
    }

    pub fn check_events(& mut self) {

        match self.event_queue.receive_timeout(Duration::from_micros(0)) {
            Some(event) => match event {
                HostEvent::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {


                        match message {
                            Message::Register(name) => {
                                if self.participants.contains_left(&name) {
                                    self.ui_sender.as_ref().unwrap().send(UiEvents::HostMessage(format!("Participant {} could not be registered. Participant with this name already exists.", name)));
                                    self.network.remove_resource(endpoint.resource_id());
                                }
                                else {
                                    self.participants.insert(name.clone(), endpoint);
                                    self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantRegistered(endpoint, name.clone()));
                                    self.ui_sender.as_ref().unwrap().send(UiEvents::ChangeStatusTo(ParticipantStatus::Idle, endpoint, name));

                                }
                            },
                            Message::Unregister => {
                                {
                                    let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                    self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantUnregistered(endpoint, endpoint_name.clone()));
                                }
                                self.participants.remove_by_right(&endpoint);
                            },
                            Message::VectorPTH(data) => {

                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();

                                self.ui_sender.as_ref().unwrap().send(UiEvents::ChangeStatusTo(ParticipantStatus::Idle, endpoint, endpoint_name.clone()));

                                if self.participants_startedwith != self.participants {
                                    self.ui_sender.as_ref().unwrap().send(UiEvents::HostMessage(format!("Some participants have disconnected/connected before execution could complete.")));

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

                                        self.ui_sender.as_ref().unwrap().send(UiEvents::InterpretResultsReturn(return_code));

                                    }
                                }
                            },
                            Message::ParticipantError(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantError(endpoint, err, endpoint_name.clone()));
                            },
                            Message::ParticipantWarning(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantWarning(endpoint, err, endpoint_name.clone()));
                            },
                            Message::Whisper(err) => {
                                let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                                self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantWhisper(endpoint, err, endpoint_name.clone()));
                            }
                            _ => {

                                panic!("Invalid message received by host ({:?})", message);
                            }
                        }
                    }
                    NetEvent::AddedEndpoint(endpoint) => {
                        //Client has connected to the host, but at this stage has not yet registered
                        println!("DEPRECIATED Client Added {}", endpoint);
                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Client disconnected without unregistering
                        {
                            let endpoint_name = self.participants.get_by_right(&endpoint).unwrap();
                            self.ui_sender.as_ref().unwrap().send(UiEvents::ParticipantUnregistered(endpoint, endpoint_name.clone()));
                        }
                        self.participants.remove_by_right(&endpoint);
                    }
                    NetEvent::DeserializationError(_) => (),
                },
                HostEvent::SendCode(code) => {
                    //println!("DEPRECIATED Sending code to all {} participant(s)", self.participants.len());
                    for (_name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Code(code.clone()));
                    }
                },
                HostEvent::SendData => {
                    //println!("DEPRECIATED Sending data to all {} participant(s)", self.participants.len());

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
                },
                HostEvent::Play(endpoint) => {
                    self.network.send(endpoint, Message::Play);
                },
                HostEvent::Stop(endpoint) => {
                    self.network.send(endpoint, Message::Stop);
                },
                HostEvent::Execute => {
                    for (name, endpoint) in self.participants.iter() {
                        self.network.send(*endpoint, Message::Execute);
                        self.ui_sender.as_ref().unwrap().send(UiEvents::ChangeStatusTo(ParticipantStatus::Calculating, *endpoint, name.clone()))
                    }
                },
                HostEvent::Begin(path) => {
                    self.start_participants(path.as_str());
                },
                HostEvent::DebugPrintCount => {
                    self.display_participant_count();
                },
                HostEvent::DebugPrintParticipants => {
                    self.display_participants();
                }
            },
            None => {

            }
        }

    }

}
