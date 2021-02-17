use crate::messages::Message;
use crate::instructions::Instructions;
use crate::operand::Operand;

use message_io::network::Endpoint;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

use stack_vm::{Machine, WriteManyTable};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Participant<'a> {
    host_endpoint: Endpoint,
    event_queue: EventQueue<Event>,
    network: Network,

    data: Option<Vec<Operand>>,

    machine: Option<Machine<'a, Operand>>,
    paused: bool,
}

impl<'a> Participant<'a> {

    pub fn new(server_address: &str) -> Self {

        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        if let Ok(host_endpoint) = network.connect(Transport::Tcp, server_address) {
            println!("Connect to server by TCP at {}", server_address);

            network.send(host_endpoint, Message::Register);

            Participant {
                host_endpoint,
                event_queue,
                network,
                data: Some(Vec::new()),
                machine: None,
                paused: false
            }
        }
        else {
            panic!("Can not connect to the server by TCP to {}", server_address);

        }


    }

    pub fn check_events(& mut self, constants: & 'a WriteManyTable<Operand>, instructions: & 'a Instructions) {

        if !self.paused {
            match &mut self.machine {
                Some(m) => {
                    match m.next() {
                        Some((instr, _args)) => {
                            //println!("calculating {}", instr.op_code);
                        },
                        None => {
                            println!("Finished");

                            let data_to_send_to_host = self.machine.take().unwrap().operand_stack.as_slice().to_vec();

                            self.network.send(self.host_endpoint, Message::VectorPTH(data_to_send_to_host));


                            self.machine = None;
                        }
                    }
                },
                None => {}
            }
        }

        match self.event_queue.receive_timeout(std::time::Duration::from_millis(10)) {
            Some(event) => match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(_, message) => {
                        match message {
                            Message::Code(code) => {

                                self.machine = Some(Machine::new(
                                    code.into(),
                                    constants,
                                    &instructions,
                                    self.data.take().unwrap())
                                );

                            },
                            Message::VectorHTP(data) => {
                                self.data = Some(data);
                            },
                            Message::Pause => {
                                self.paused = true;
                            },
                            Message::Play => {
                                self.paused = false;
                            },
                            Message::Stop => {
                                self.machine = None;
                                self.paused = false;
                            }
                            _ => { /*panic!("Invalid message {:?}", message);*/ }
                        }
                    }
                    NetEvent::AddedEndpoint(_endpoint) => {},
                    NetEvent::RemovedEndpoint(_endpoint) => {
                        println!("Server Disconnected");
                    }
                    NetEvent::DeserializationError(_) => (),
                },
            },
            None => {

            }
        }



    }

}
