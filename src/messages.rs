
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /* Host to participant */

    Code(String),
    VectorHTP(Vec<f64>),
    Execute,

    Play,
    Pause,
    Stop,

    /* Participant to Host */

    VectorPTH(Vec<f64>),

    Progress(f32),

    Register,
    Unregister
}