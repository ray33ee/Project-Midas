
use crate::messages::{UiEvents, HostEvent, ParticipantStatus, Severity, NodeType};
use std::time::Duration;

use crossbeam_channel::{Sender, Receiver};

use crossterm::event::{read, Event, poll};


use std::io;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, ListItem, List, Row, Table, Cell, ListState, Paragraph};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style};
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

    selected_participant: Option<String>,

    participants_state: ListState,

    top_scroll_log_item: usize, //Index of the log item to show at the top



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
            selected_participant: None,
            participant_names: Vec::new(),
            top_scroll_log_item: 0,
            logs: Vec::new()
        }
    }

    fn is_calculating(& self) -> bool {
        let mut is_calculating = false;

        for info in self.participants.right_values().into_iter() {
            is_calculating |= info.status != ParticipantStatus::Idle;
        }

        is_calculating
    }

    pub fn tick(& mut self) -> Result<(), ()> {

        //If no participant is selected, try and select one
        if !self.participant_names.is_empty() && self.selected_participant == Option::None {
            self.selected_participant = Some(self.participant_names.get(0).unwrap().clone());
            self.participants_state.select(Some(0));
        }

        //When a button is clicked or an action is invoked, we must send the event via the ui_sender
        if let Ok(true) = poll(Duration::from_secs(0)) {
            match read().unwrap() {
                Event::Key(key_event) => {
                    match key_event.code {
                        crossterm::event::KeyCode::Char('e') => {

                            if !self.is_calculating() {
                                self.command_sender.send(HostEvent::Begin(String::from(self.script_path))).unwrap();

                            }

                        },
                        crossterm::event::KeyCode::Char('p') => {
                            self.command_sender.send(HostEvent::PauseAll).unwrap();
                        },
                        crossterm::event::KeyCode::Char('l') => {
                            self.command_sender.send(HostEvent::PlayAll).unwrap();
                        },
                        crossterm::event::KeyCode::Char('k') => {
                            self.command_sender.send(HostEvent::KillAll).unwrap();
                            self.logs.insert(0, LogEntry::new(Severity::Info, NodeType::Host, format!("Terminating all participants.")));

                        },
                        crossterm::event::KeyCode::Char('q') => {


                            self.command_sender.send(HostEvent::RemoveAll).unwrap();
                            self.terminal.clear().unwrap();
                            return Err(());
                        },
                        crossterm::event::KeyCode::Char('c') => {
                            self.logs.clear();
                            self.top_scroll_log_item = 0;
                        },
                        crossterm::event::KeyCode::Left => {


                        },
                        crossterm::event::KeyCode::Right => {

                        },
                        crossterm::event::KeyCode::Up => {

                            if !self.participants.is_empty() {

                                let selected_index = self.participants_state.selected().unwrap();

                                if selected_index != 0 {
                                    self.participants_state.select(Some(selected_index - 1));
                                } else {
                                    self.participants_state.select(Some(self.participant_names.len() - 1));
                                }
                            }
                        },
                        crossterm::event::KeyCode::Down => {
                            if !self.participants.is_empty() {
                                let selected_index = self.participants_state.selected().unwrap();

                                self.participants_state.select(Some(selected_index + 1));
                            }
                        },
                        crossterm::event::KeyCode::PageUp => {
                            if self.top_scroll_log_item < 10 {
                                self.top_scroll_log_item = 0;
                            } else {
                                self.top_scroll_log_item -= 10;

                            }


                        },
                        crossterm::event::KeyCode::PageDown => {
                            if self.top_scroll_log_item + 10 < self.logs.len() {
                                self.top_scroll_log_item += 10;
                            }

                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        self.selected_participant = match self.participants_state.selected() {
            Some(index) => match self.participant_names.get(index) {
                Some(name) => Some(name.clone()),
                None => None
            }
            None => None
        };

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

                    self.logs.insert(0, LogEntry::new(severity, node_type, message));
                }

                UiEvents::ParticipantRegistered(endpoint, name) => {
                    self.participants.insert(name, ParticipantInfo::new(endpoint));
                },
                UiEvents::ParticipantUnregistered( name) => {

                    self.logs.insert(0, LogEntry::new(Severity::Warning, NodeType::Participant(name.clone()), format!("Participant has disconnected.")));
                    self.participants.remove_by_left(&name);
                },
                UiEvents::InterpretResultsReturn(return_message) => {
                    self.logs.insert(0, LogEntry::new(Severity::Result, NodeType::Host, return_message));
                },
                UiEvents::ParticipantProgress(name, progress) => {
                    let (_, mut info) = self.participants.remove_by_left(&name).unwrap();
                    info.progress = Some((progress * 100.0f32) as i32);
                    self.participants.insert(name, info);
                }
            },
            Err(_e) => {

            }
        }


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
        //let state = & mut self.participants_state;

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
                            Spans::from(format!("Progress: {}",
                                                match info.progress {
                                                    Some(number) => format!("{}%", number as f32 / 100.0f32),
                                                    None => format!("-")
                                                }
                            )),
                        ])
                    }
                    None => Text::raw("")
                }
            }
            None => {
                Text::raw("")
            }
        };


        let top_scroll = self.top_scroll_log_item;

        let messages_items: Vec<_> = self.logs.iter().map(|entry| {
            entry.to_listitem()
        }).collect();


        //Generate the UI
        self.terminal.draw(|f| {
            let v_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(19),
                        Constraint::Length(1),
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

                .highlight_style(Style::default()
                    .bg(Color::Rgb(50, 50, 50)))
                .highlight_symbol("");



            let messages_slice = &messages_items[top_scroll..];



            f.render_stateful_widget(participant_list, h_chunks[0], state);

            let log_table = Table::new(messages_slice.to_vec())
                .header(
                    Row::new(vec!["Time", "Level", "Target", "Message"])
                        .style(Style::default().fg(Color::Rgb(229, 228, 226)))
                        .bottom_margin(1)
                )
                .widths(&[Constraint::Length(19), Constraint::Length(9), Constraint::Length(12), Constraint::Length(500)])
                .column_spacing(1)
                .block(Block::default().title("Logs").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));

            f.render_widget(log_table, h_chunks[1]);

            let info = Paragraph::new(text.clone())
                .block(Block::default().title("Info").borders(Borders::ALL))
                ;
            f.render_widget(info, v_chunks[1]);

            let shortcuts = Paragraph::new(Text::from(vec![Spans::from(vec![
                Span::raw("q "),
                Span::styled("Exit             ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("e "),
                Span::styled("Execute          ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("p "),
                Span::styled("Pause            ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("l "),
                Span::styled("Play             ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("k "),
                Span::styled("Kill             ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("c "),
                Span::styled("Clear Log        ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("PgDn "),
                Span::styled("Scroll down log  ", Style::default().fg(Color::Rgb(58, 47, 77))),
                Span::raw("PgUp "),
                Span::styled("Scroll up log    ", Style::default().fg(Color::Rgb(58, 47, 77))),
            ])])).block(Block::default());

            f.render_widget(shortcuts, v_chunks[2]);

        }).unwrap();
        self.terminal.autoresize().unwrap();


        Ok(())

    }
}

