use crate::messages::Message;
use crate::instructions::SerdeCodeOperand;

use message_io::network::Endpoint;

use std::collections::HashSet;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};
use crate::instructions::get_instructions;

use crate::operand::Operand;

enum Event {
    Network(NetEvent<Message>),
    SendCode(Endpoint, SerdeCodeOperand),
    SendData(Endpoint, Vec<Operand>),
    Pause(Endpoint),
    Play(Endpoint),
    Stop(Endpoint),
}

pub struct Host {
    participants: HashSet<Endpoint>,
    event_queue: EventQueue<Event>,
    network: Network
}

impl Host {

    pub fn new(server_address: &str) -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        match network.listen(Transport::Tcp, server_address) {
            Ok(_) => println!("TCP Server running at {}", server_address),
            Err(_) => panic!("Can not listening at {}", server_address)
        }

        Host {
            participants: HashSet::new(),
            event_queue,
            network
        }
    }

    pub fn test_participant_event(& mut self, prime: i64) {

        let participant_count = self.participants.len();

        if participant_count == 1 {
            use crate::compiler::Compiler;

            let table = get_instructions();

            let upper: usize = (prime as f64).sqrt() as usize;
            let width: usize = (upper - 1) / participant_count;

            println!("upp: {}", upper - 1);

            //get the code from a file
            let (builder, cons) = Compiler::compile_file(".\\docs\\sample_code.txt", &table);

            for (i, endpoint) in self.participants.iter().enumerate() {
                if i == participant_count - 1
                {
                    self.event_queue.sender().send(Event::SendData(*endpoint,
                                                                   vec![Operand::I64(prime), Operand::I64((2 + width * i) as i64), Operand::I64((2 + width * (i + 1)) as i64)]
                    ));
                } else {
                    self.event_queue.sender().send(Event::SendData(*endpoint,
                                                                   vec![Operand::I64(prime), Operand::I64((2 + width * i) as i64), Operand::I64((2 + width * (i + 1) + ((upper - 1) % participant_count)) as i64)]
                    ));
                }


                self.event_queue.sender().send(
                    Event::SendCode(*endpoint, SerdeCodeOperand::from(builder.clone())));
            }
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

                            self.test_participant_event(		413158511);

                            println!("Set: {:?}", self.participants);
                        },
                        Message::Unregister => {
                            self.participants.remove(&endpoint);
                        },
                        Message::VectorPTH(data) => {
                            //Save data to computer path using endpoint and time and date as file name

                            println!("Data received: {:?}", data);
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
            Event::SendCode(endpoint, code) => {
                self.network.send(endpoint, Message::Code(code));
            },
            Event::SendData(endpoint, data) => {
                self.network.send(endpoint, Message::VectorHTP(data));
            },
            Event::Pause(endpoint) => {
                self.network.send(endpoint, Message::Pause);
            },
            Event::Play(endpoint) => {
                self.network.send(endpoint, Message::Play);
            },
            Event::Stop(endpoint) => {
                self.network.send(endpoint, Message::Stop);
            }
        }



    }

}
