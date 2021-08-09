use std::time::Duration;

pub struct SentPacket {
    pub index: u64,
    pub sent_duration: Duration,
    pub sent_timestamp: Duration
}

pub struct ReceivedPacket {
    pub index: u64,
    pub received_duration: Duration,
    pub received_timestamp: Duration,
    pub server_timestamp: Duration
}
