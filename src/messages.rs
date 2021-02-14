
use crate::instructions::SerdeCode;
use crate::operand::Operand;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /* Host to participant */

    Code(SerdeCode<Operand>),
    VectorHTP(Vec<Operand>),

    Play,
    Pause,
    Stop,

    /* Participant to Host */

    VectorPTH(Vec<Operand>),

    Register,
    Unregister
}