
use crate::instructions::SerdeCode;
use crate::operand::Operand;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /* Host to participant */

    Code(SerdeCode<Operand>),
    VectorI64HTP(Vec<i64>),
    VectorF64HTP(Vec<f64>),

    Play,
    Pause,
    Stop,

    /* Participant to Host */

    VectorI64PTH(Vec<i64>, i64),
    VectorF64PTH(Vec<f64>, i64),

    Register,
    Unregister
}