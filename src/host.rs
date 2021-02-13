use crate::messages::Message;
use crate::instructions::SerdeCodeOperand;

use message_io::network::Endpoint;

use std::collections::HashSet;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};
use crate::instructions::get_instructions;

enum Event {
    Network(NetEvent<Message>),
    SendCode(Endpoint, SerdeCodeOperand),
    SendI64(Endpoint, Vec<i64>),
    SendF649(Endpoint, Vec<f64>)
}

pub struct Host {
    participants: HashSet<Endpoint>,
    event_queue: EventQueue<Event>,
    network: Network
}

impl Host {

    pub fn new() -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let server_address = "127.0.0.1:3000";

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

    pub fn test_event(& mut self) {

        use crate::operand::Operand;

        let table = get_instructions();
        let mut builder = stack_vm::Builder::new(&table);
        builder.push("pushl", vec![Operand::I64(40)]);
        builder.push("print_s", vec![]);

        self.event_queue.sender().send(Event::SendCode(*self.participants.iter().next().unwrap(), SerdeCodeOperand::from(builder)));

    }

    pub fn check_events(& mut self) {


        match self.event_queue.receive() {
            Event::Network(net_event) => match net_event {
                NetEvent::Message(endpoint, message) => {

                    println!("Message got");

                    match message {
                        Message::Register => {
                            println!("    Register participant");
                            self.participants.insert(endpoint);


                            self.test_event();

                            println!("Set: {:?}", self.participants);
                        },
                        Message::Unregister => {
                            self.participants.remove(&endpoint);
                        },
                        Message::VectorI64PTH(data, identifier) => {
                            //Save data to computer path using 'identifier' and time and date as file name
                        },
                        Message::VectorF64PTH(data, identifier) => {
                            //Save data to computer path using 'identifier' and time and date as file name

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
            Event::SendI64(endpoint, data) => {
                self.network.send(endpoint, Message::VectorI64HTP(data));
            },
            Event::SendF649(endpoint, data) => {
                self.network.send(endpoint, Message::VectorF64HTP(data));
            }
        }



    }

}
