
use serde::{Serialize, Deserialize};

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

    Register,
    Unregister
}