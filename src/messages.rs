use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitMessage {
    pub packet_size: usize
}
