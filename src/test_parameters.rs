use std::time::Duration;

#[derive(Clone)]
pub struct TestParameters {
    pub server_ip: String,
    pub server_port: u16,
    pub protocol: String,
    pub time: Duration,
    pub packets_per_second: u32,
    pub packet_size: usize,
    pub output_rtt: bool,
    pub measure_owl: bool
}
