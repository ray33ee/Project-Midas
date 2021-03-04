
use serde::{Serialize, Deserialize};
use message_io::network::Endpoint;
use message_io::network::NetEvent;

use crate::lua::SerdeLuaTable;
use tui::style::{Style, Color, Modifier};
use tui::widgets::Cell;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /* Host to participant */

    Code(String),
    VectorHTP(SerdeLuaTable),
    Execute,

    Play,
    Pause,
    Stop,

    /* Participant to Host */

    VectorPTH(SerdeLuaTable),

    Progress(f32),

    ParticipantError(String),
    ParticipantWarning(String),
    Whisper(String),
    Stdout(String),

    Paused,
    Continued,
    Executing,

    Register(String),
    Unregister
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ParticipantStatus {
    Idle,
    Calculating,
    Paused,

}

impl ParticipantStatus {
    pub fn to_color(& self) -> Color {
        match self {
            ParticipantStatus::Idle => Color::Green,
            ParticipantStatus::Calculating => Color::Rgb(255, 255, 0),
            ParticipantStatus::Paused => Color::Rgb(255, 128, 0),
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Result,
    Stdout
}

impl Severity {
    pub fn to_cell(& self) -> Cell {
        match self {
            Severity::Info => Cell::from("INFO").style(Style::default()),
            Severity::Warning => Cell::from("WARNING").style(Style::default().fg(Color::Rgb(255, 255, 0))),
            Severity::Error => Cell::from("ERROR").style(Style::default().fg(Color::Rgb(255, 0, 0))),
            Severity::Result => Cell::from("RESULT").style(Style::default().fg(Color::Rgb(255, 0, 100))),
            Severity::Stdout => Cell::from("STDOUT").style(Style::default().fg(Color::Rgb(0, 255, 128))),
        }
    }
}

pub enum NodeType {
    Host,
    Participant(String),
}

impl NodeType {
    pub fn to_cell(& self) -> Cell {
        match self {
            NodeType::Host => Cell::from("Host").style(Style::default().fg(Color::Rgb(0, 255, 255))),
            NodeType::Participant(name) => Cell::from(name.as_str()).style(Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::ITALIC)),
        }
    }
}

pub enum UiEvents {
    ChangeStatusTo(ParticipantStatus, Endpoint, String),

    ParticipantProgress(Endpoint, String, f32),

    Log(NodeType, String, Severity),

    /*ParticipantError(Endpoint, String, String),
    ParticipantWarning(Endpoint, String, String),
    ParticipantWhisper(Endpoint, String, String),*/

    ParticipantRegistered(Endpoint, String),
    ParticipantUnregistered(Endpoint, String),

    InterpretResultsReturn(String),

}


pub enum HostEvent {
    Network(NetEvent<Message>),
    SendCode(String),
    SendData,
    Pause(Endpoint),
    Play(Endpoint),
    Kill(Endpoint),
    Execute,

    Begin(String),

    PlayAll,
    PauseAll,
    KillAll,
}