
use crate::messages::{ParticipantStatus, UiEvents, HostEvent};
use message_io::events::{EventQueue, EventSender};
use std::time::Duration;

pub struct Panel {
    //For events sent from the host
    ui_event_queue: EventQueue<UiEvents>,

    //For events sent to the host
    ui_sender: EventSender<HostEvent>
}

impl Panel {
    pub fn new(ui_sender: EventSender<HostEvent>) -> (EventSender<UiEvents>, Self) {

        let mut ui_event_queue = EventQueue::new();

        ( ui_event_queue.sender().clone(),
        Panel {
            ui_event_queue,
            ui_sender
        })
    }

    pub fn tick(& mut self) {
        //When a button is clicked or an action is invoked, we must send the event via the ui_sender

        //We must also check ui_event_queue and see if we need to change the UI
        match self.ui_event_queue.receive_timeout(Duration::from_micros(0)) {
            Some(event) => match event {
                UiEvents::ChangeStatusTo(status, _endpoint, name) => match status {
                    ParticipantStatus::Calculating => {
                        println!("Participant '{}' is calculating. Awaiting results.", name);
                    }
                    ParticipantStatus::Idle => {
                        println!("Participant '{}' is now idle.", name);
                    }
                },
                UiEvents::ParticipantError(_endpoint, error, name) => {
                    println!("Participant '{}' Error: {}", name, error);
                },
                UiEvents::ParticipantWarning(_endpoint, warning, name) => {
                    println!("Participant '{}' Warning: {}", name, warning);
                },
                UiEvents::ParticipantWhisper(_endpoint, message, name) => {
                    println!("Participant '{}' Whisper: {}", name, message);
                },
                UiEvents::ParticipantRegistered(endpoint, name) => {

                    println!("Client '{}' has connected. (endpoint: {})", name, endpoint);
                },
                UiEvents::ParticipantUnregistered(endpoint, name) => {

                    println!("Client '{}' has disconnected. (endpoint: {})", name, endpoint);
                },
                UiEvents::HostMessage(message) => {
                    println!("Host message - {}", message);
                },
                UiEvents::InterpretResultsReturn(return_message) => {
                    println!("All participants finished. interpret_results return code: {}", return_message);
                }
            },
            None => {}
        }

        //Generate the UI
    }

    /*fn get_sender(& mut self) -> EventSender<UiEvents> {
        self.event_queue.sender().clone()
    }*/
}

