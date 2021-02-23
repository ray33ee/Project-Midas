
use serde::{Serialize, Deserialize};
use message_io::network::Endpoint;
use message_io::network::NetEvent;

use crate::lua::SerdeLuaTable;

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

pub enum ParticipantStatus {
    Idle,
    Calculating,
}

pub enum UiEvents {
    ChangeStatusTo(ParticipantStatus, Endpoint, String),

    ParticipantError(Endpoint, String, String),
    ParticipantWarning(Endpoint, String, String),
    ParticipantWhisper(Endpoint, String, String),

    ParticipantRegistered(Endpoint, String),
    ParticipantUnregistered(Endpoint, String),

    InterpretResultsReturn(String),

    HostMessage(String),

}


pub enum HostEvent {
    Network(NetEvent<Message>),
    SendCode(String),
    SendData,
    Pause(Endpoint),
    Play(Endpoint),
    Stop(Endpoint),
    Execute,

    Begin(String),

    DebugPrintCount,
    DebugPrintParticipants,
}