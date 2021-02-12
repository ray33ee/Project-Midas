use crate::messages::Message;

use message_io::network::Endpoint;

use std::collections::HashSet;

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Transport};

enum Event {
    Network(NetEvent<Message>)
}

pub struct Host {
    participants: HashSet<Endpoint>
}

impl Host {

    pub fn new() -> Self {
        Host {
            participants: HashSet::new()
        }
    }

    pub fn run(&self) {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();

        let mut network = Network::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let server_address = "127.0.0.1:3000";

        match network.listen(Transport::Tcp, server_address) {
            Ok(_) => println!("TCP Server running at {}", server_address),
            Err(_) => return println!("Can not listening at {}", server_address)
        }


        loop {
            match event_queue.receive() {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {

                        println!("Message got");


                    }
                    NetEvent::AddedEndpoint(endpoint) => {
                        println!("Client Added {:?}", endpoint);
                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        //Client disconnected without unregistering
                        println!("Client Disconnected {:?}", endpoint);
                    }
                    NetEvent::DeserializationError(_) => (),
                },
            }
        }


    }

}
