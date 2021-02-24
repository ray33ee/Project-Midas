
use crate::messages::{ParticipantStatus, UiEvents, HostEvent};
use std::time::Duration;

use std::sync::mpsc::{Sender, Receiver};

use crossterm::event::{read, Event, poll};

pub struct Panel {
    //For events sent from the host
    //ui_event_queue: EventQueue<UiEvents>,

    //For events sent to the host
    //ui_sender: EventSender<HostEvent>

    command_sender: Sender<HostEvent>,

    message_receiver: Receiver<UiEvents>,

    script_path: String,
}

impl Panel {
    pub fn new(command_sender: Sender<HostEvent>, message_receiver: Receiver<UiEvents>, script_path: String) -> Self {

        //let mut ui_event_queue = EventQueue::new();

        //let (message_sender, message_receiver) = channel();


        Panel {
            command_sender,
            message_receiver,
            script_path
        }
    }

    pub fn tick(& mut self) {
        //When a button is clicked or an action is invoked, we must send the event via the ui_sender
        if let Ok(true) = poll(Duration::from_secs(0)) {
            match read().unwrap() {
                Event::Key(key_event) => {
                    match key_event.code {
                        crossterm::event::KeyCode::Char('e') => {
                            //host.start_participants(script_path);
                            self.command_sender.send(HostEvent::Begin(self.script_path.clone())).unwrap();
                        },
                        crossterm::event::KeyCode::Char('d') => {
                            //host.display_participants();
                            self.command_sender.send(HostEvent::DebugPrintParticipants).unwrap();
                        },
                        crossterm::event::KeyCode::Char('c') => {
                            //host.display_participant_count();
                            self.command_sender.send(HostEvent::DebugPrintCount).unwrap();
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }


        //We must also check ui_event_queue and see if we need to change the UI
        match self.message_receiver.recv_timeout(Duration::from_micros(0)) {
            Ok(event) => match event {
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
            Err(_e) => {
                //println!("Receive error in panel - {}", e);
            }
        }

        //Generate the UI
    }

    /*fn get_sender(& mut self) -> EventSender<UiEvents> {
        self.event_queue.sender().clone()
    }*/
}

