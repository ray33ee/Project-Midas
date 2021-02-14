use crate::messages::Message;
use crate::instructions::{get_instructions, Instructions, get_constants};
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
    constants: WriteManyTable<Operand>,

    data: Vec<Operand>
}

impl Participant {

    pub fn new(server_address: &str) -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let constants = get_constants();

        if let Ok(host_endpoint) = network.connect(Transport::Tcp, server_address) {
            println!("Connect to server by TCP at {}", server_address);


            network.send(host_endpoint, Message::Register);

            Participant {
                host_endpoint,
                event_queue,
                network,
                instructions: get_instructions(),
                constants,
                data: Vec::new()
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

                    println!("Message got");

                    match message {
                        Message::Code(code) => {

                            let mut machine = Machine::new(code.into(), &self.constants, &self.instructions);

                            //machine.set_local("0", Operand::I64(55));

                            //Push data sent from host onto the local variables in the VM

                            for (i, operand) in self.data.iter().enumerate() {
                                machine.set_local(&format!("d_{}", i.to_string()), *operand);
                            }

                            machine.run();

                            let data_to_send_to_host = machine.operand_stack.as_slice().to_vec();


                            self.network.send(self.host_endpoint, Message::VectorPTH(data_to_send_to_host));


                        },
                        Message::VectorHTP(data) => {
                            self.data = data;
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
