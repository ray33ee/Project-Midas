
use serde::{Serialize, Deserialize};
use message_io::network::Endpoint;
use message_io::network::NetEvent;

use crate::lua::SerdeLuaTable;
use tui::style::{Style, Color};
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

    Register(String),
    Unregister
}
#[derive(PartialEq, Eq, Hash)]
pub enum ParticipantStatus {
    Idle,
    Calculating,
    Paused,

}
#[derive(PartialEq, Eq, Hash)]
pub enum Severity {
    Whisper,
    Warning,
    Error
}

impl Severity {
    pub fn to_cell(& self) -> Cell {
        match self {
            Severity::Whisper => Cell::from("INFO").style(Style::default()),
            Severity::Warning => Cell::from("WARNING").style(Style::default().fg(Color::Rgb(255, 255, 0))),
            Severity::Error => Cell::from("ERROR").style(Style::default().fg(Color::Rgb(255, 0, 0))),
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
            NodeType::Participant(name) => Cell::from(name.as_str()).style(Style::default().fg(Color::Rgb(255, 0, 255))),
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