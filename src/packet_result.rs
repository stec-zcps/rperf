#[derive(Copy, Clone)]
pub struct PacketResult {
    pub index: u64,
    pub tx_time: f64,
    pub rx_time: f64,
    pub latency: f64,
    pub latency_client_to_server: f64,
    pub latency_server_to_client: f64
}
