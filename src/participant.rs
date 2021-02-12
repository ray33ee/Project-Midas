use crate::messages::Message;

use message_io::network::Endpoint;

use std::collections::HashSet;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Participant {

}

impl Participant {

    pub fn new() -> Self {
        Participant {

        }
    }

    pub fn run(&self) {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let server_address = "127.0.0.1:3000";

        if let Ok(server_id) = network.connect(Transport::Tcp, server_address) {
            println!("Connect to server by TCP at {}", server_address);

            loop {

            }

            /*loop {
                match event_queue.receive() {
                    Event::Network(net_event) => match net_event {
                        NetEvent::Message(endpoint, message) => {

                            println!("Message got");

                            match message {
                                Message::Greetings(text) => println!("Server says: {}", text),
                            }
                        }
                        NetEvent::AddedEndpoint(_) => {
                            println!("Client Added");
                        },
                        NetEvent::RemovedEndpoint(endpoint) => {
                            //Client disconnected without unregistering
                            println!("Client Disconnected");
                        }
                        NetEvent::DeserializationError(_) => (),
                    },
                }
            }*/
        }
        else {
            println!("Can not connect to the server by TCP to {}", server_address);
        }

    }

}
