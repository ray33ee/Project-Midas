
use crate::messages::{UiEvents, HostEvent, ParticipantStatus, Severity, NodeType};
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver};

use crossterm::event::{read, Event, poll};


use std::io;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, ListItem, List, Row, Table, Cell, ListState, Paragraph};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Modifier, Color, Style};
use tui::text::{Spans, Span};


use std::io::Stdout;
use bimap::BiMap;
use message_io::network::Endpoint;

use chrono::{Utc, DateTime};
use tui::text::Text;

#[derive(PartialEq, Eq, Hash, Clone)]
struct ParticipantInfo {
    endpoint: Endpoint,
    status: ParticipantStatus,
    progress: Option<i32>,
}

impl ParticipantInfo {
    fn new(endpoint: Endpoint) -> Self {

        ParticipantInfo {
            endpoint,
            status: ParticipantStatus::Idle,
            progress: None,
        }
    }
}

struct LogEntry {
    severity: Severity,
    node_type: NodeType,
    message: String,
    time: DateTime<Utc>,
}

impl LogEntry {
    fn new(severity: Severity, node_type: NodeType, message: String) -> Self {
        LogEntry {
            time: Utc::now(),
            severity,
            node_type,
            message
        }
    }

    fn to_listitem(& self) -> Row {

        Row::new(vec![
            Cell::from(format!("{}", self.time.format("%F %X"))),
            self.severity.to_cell(),
            self.node_type.to_cell(),
            Cell::from(self.message.as_str()),
        ])
    }
}



pub struct Panel<'a> {
    command_sender: Sender<HostEvent>,

    message_receiver: Receiver<UiEvents>,

    terminal: Terminal<CrosstermBackend<Stdout>>,

    script_path: & 'a str,

    participants: BiMap<String, ParticipantInfo>,

    participant_names: Vec<String>,

    selected_widget: i32,

    selected_participant: Option<String>,

    participants_state: ListState,

    logs: Vec<LogEntry>,

}

impl<'a> Panel<'a> {
    pub fn new(command_sender: Sender<HostEvent>, message_receiver: Receiver<UiEvents>, script_path: & 'a str) -> Self {

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        Panel {
            command_sender,
            message_receiver,
            terminal,
            script_path,
            participants: BiMap::new(),
            participants_state: ListState::default(),
            selected_widget: 0,
            selected_participant: None,
            participant_names: Vec::new(),
            logs: Vec::new()
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
                            self.command_sender.send(HostEvent::Begin(String::from(self.script_path))).unwrap();
                        },
                        crossterm::event::KeyCode::Char('p') => {
                            //host.display_participant_count();
                            //println!("Pause");
                            self.command_sender.send(HostEvent::PauseAll).unwrap();
                        },
                        crossterm::event::KeyCode::Char('l') => {
                            //host.display_participant_count();
                            //println!("Play");
                            self.command_sender.send(HostEvent::PlayAll).unwrap();
                        },
                        crossterm::event::KeyCode::Char('s') => {
                            //host.display_participant_count();
                            //println!("Stop");
                            self.command_sender.send(HostEvent::KillAll).unwrap();
                        },
                        crossterm::event::KeyCode::Char('q') => {
                            //host.display_participant_count();
                            //println!("Stop");
                            self.terminal.clear().unwrap();
                            panic!("Ending");
                        },
                        crossterm::event::KeyCode::Left => {
                            self.selected_widget = 0;

                        },
                        crossterm::event::KeyCode::Right => {
                            self.selected_widget = 1;
                        },
                        crossterm::event::KeyCode::Up => {
                            match self.selected_widget {
                                0 => {
                                    let selected_index = self.participants_state.selected().unwrap_or(0);

                                    if !self.participants.is_empty() {
                                        if selected_index != 0 {
                                            self.participants_state.select(Some(selected_index - 1));
                                        }
                                        else {
                                            self.participants_state.select(Some(selected_index));
                                        }
                                    }
                                },
                                1 => {

                                }
                                _ => {}
                            }
                        },
                        crossterm::event::KeyCode::Down => {
                            match self.selected_widget {
                                0 => {
                                    let selected_index = self.participants_state.selected().unwrap_or(0);
                                    if !self.participants.is_empty() {
                                        if selected_index != self.participants.len() - 1 {
                                            self.participants_state.select(Some(selected_index + 1));
                                        }
                                    }
                                },
                                1 => {

                                }
                                _ => {}
                            }
                        },
                        crossterm::event::KeyCode::Enter => {
                            match self.selected_widget {
                                0 => {
                                    self.selected_participant = match self.participants_state.selected() {
                                        Some(index) => {
                                            Some(self.participant_names.get(index).unwrap().clone())
                                        }
                                        None => None
                                    }
                                },
                                _ => {}
                            }
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        }


        //We must also check ui_event_queue and see if we need to change the UI
        match self.message_receiver.recv_timeout(Duration::from_micros(0)) {
            Ok(event) => match event {
                UiEvents::ChangeStatusTo(status, _endpoint, name) => {
                    let (_, mut info) = self.participants.remove_by_left(&name).unwrap();
                    if let ParticipantStatus::Idle = status {
                        info.progress = None;
                    }
                    info.status = status;
                    self.participants.insert(name, info);

                },
                UiEvents::Log(node_type, message, severity) => {
                    /*let (_, mut info) = self.participants.remove_by_left(&name).unwrap();
                    info.logs.insert(0, (severity, message));
                    self.participants.insert(name, info);*/

                    self.logs.insert(0, LogEntry::new(severity, node_type, message));
                }

                UiEvents::ParticipantRegistered(endpoint, name) => {
                    self.participants.insert(name, ParticipantInfo::new(endpoint));
                    //println!("Client '{}' has connected. (endpoint: {})", name, endpoint);
                },
                UiEvents::ParticipantUnregistered(endpoint, name) => {

                    self.logs.insert(0, LogEntry::new(Severity::Info, NodeType::Participant(name.clone()), format!("Client has disconnected.")));
                    self.participants.remove_by_left(&name);
                    //println!("Client '{}' has disconnected. (endpoint: {})", name, endpoint);
                },
                UiEvents::InterpretResultsReturn(return_message) => {
                    //println!("All participants finished. interpret_results return code: {}", return_message);
                    self.logs.insert(0, LogEntry::new(Severity::Result, NodeType::Host, return_message));
                },
                UiEvents::ParticipantProgress(endpoint, name, progress) => {
                    let (_, mut info) = self.participants.remove_by_left(&name).unwrap();
                    info.progress = Some(progress as i32);
                    self.participants.insert(name, info);
                    //println!("Client '{}' Progress: {}. (endpoint: {})", name, progress, endpoint);
                }
            },
            Err(_e) => {
                //println!("Receive error in panel - {}", e);
            }
        }

        let messages_items: Vec<_> = self.logs.iter().map(|entry| {
              entry.to_listitem()
          }).collect();

        self.participant_names = self.participants.iter()
            .map(|(string, _)| string.clone())
            .collect()
            ;

        self.participant_names.sort();

        let participant_items: Vec<_> = self.participant_names.iter()
            .map(|string| ListItem::new(string.as_str())
                .style(Style::default()
                    .fg(
                        self.participants.get_by_left(string).unwrap().status.to_color()
                    )))
            .collect();


        let state = & mut self.participants_state;

        let text = match &self.selected_participant {
            Some(name) => {
                match self.participants.get_by_left(name) {
                    Some(info) => {
                        Text::from(vec![
                            Spans::from(format!("Name:     {}", name)),
                            Spans::from(format!("Endpoint: {}", info.endpoint)),
                            Spans::from(vec![/*format!("Progress: {:?}", info.progress)*/
                                             Span::raw("Status:   "),
                                             Span::styled(format!("{:?}", info.status), Style::default().fg(info.status.to_color()))
                            ]),
                            Spans::from(format!("Progress: {:?}", info.progress)),
                        ])
                    }
                    None => Text::raw("naut")
                }
            }
            None => {
                Text::raw("naut")
            }
        };

        //Generate the UI
        self.terminal.draw(|f| {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20),
                    ].as_ref()
                )
                .split(f.size());


            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(80)
                    ].as_ref()
                )
                .split(v_chunks[0]);


            let participant_list = List::new(participant_items)
                .block(Block::default().title("Participants").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");


            f.render_stateful_widget(participant_list, h_chunks[0], state);


            let log_table = Table::new(messages_items)
                .header(
                    Row::new(vec!["Time", "Level", "Target", "Message"])
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1)
                )
                .widths(&[Constraint::Length(19), Constraint::Length(7), Constraint::Length(12), Constraint::Length(500)])
                .column_spacing(1)
                .block(Block::default().title("Logs").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>").style(Style::default().fg(Color::White));



            f.render_widget(log_table, h_chunks[1]);



            let block = Paragraph::new(text)
                .block(Block::default().title("Info").borders(Borders::ALL))
                ;
            f.render_widget(block, v_chunks[1]);



        }).unwrap();
        self.terminal.autoresize().unwrap();

    }
}

