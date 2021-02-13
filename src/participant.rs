use crate::messages::Message;
use crate::instructions::{get_instructions, Instructions};
use crate::operand::Operand;

use message_io::network::Endpoint;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

use stack_vm::{Machine, WriteManyTable};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Participant {
    host_endpoint: Endpoint,
    event_queue: EventQueue<Event>,
    network: Network,

    instructions: Instructions,
    constants: WriteManyTable<Operand>
}

impl Participant {

    pub fn new() -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let server_address = "127.0.0.1:3000";

        if let Ok(host_endpoint) = network.connect(Transport::Tcp, server_address) {
            println!("Connect to server by TCP at {}", server_address);


            network.send(host_endpoint, Message::Register);

            Participant {
                host_endpoint,
                event_queue,
                network,
                instructions: get_instructions(),
                constants: WriteManyTable::new()
            }
        }
        else {
            panic!("Can not connect to the server by TCP to {}", server_address);

        }


    }

    pub fn check_events(& mut self) {




        match self.event_queue.receive() {
            Event::Network(net_event) => match net_event {
                NetEvent::Message(endpoint, message) => {

                    println!("Message got");

                    match message {
                        Message::Code(code) => {

                            let mut machine = Machine::new(code.into(), &self.constants, &self.instructions);
                            machine.run();

                        },
                        Message::VectorI64HTP(data) => {

                        },
                        Message::VectorF64HTP(data) => {

                        },
                        _ => { panic!("Invalid message {:?}", message); }
                    }
                }
                NetEvent::AddedEndpoint(_endpoint) => {

                },
                NetEvent::RemovedEndpoint(_endpoint) => {
                    println!("Server Disconnected");
                }
                NetEvent::DeserializationError(_) => (),
            },
        }



    }

}
