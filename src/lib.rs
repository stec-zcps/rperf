mod server_udp;
mod server_tcp;
mod client;
mod test_parameters;
mod test_result;
mod messages;
mod packet;
mod packet_result;

use crate::client::client::Client;
use std::time::Duration;
use std::io::ErrorKind;
use crate::test_result::TestResult;

pub async fn start_server(port: u16, protocol: &str, symmetric_network_load: bool) -> std::io::Result<()>
{
    match protocol.to_lowercase().as_str() {
        "udp" => {
            let server = server_udp::server::ServerUdp::new();
            server.start(port, symmetric_network_load).unwrap_or_else(|error| {
                panic!("Problem running test: {:?}", error);
            });
            Ok(())
        },
        "tcp" => {
            let server = server_tcp::server::ServerTcp::new();
            server.start(port, symmetric_network_load).await.unwrap_or_else(|error| {
                panic!("Problem running test: {:?}", error);
            });
            Ok(())
        },
        _ => {
            println!("Unsupported protocol '{}' ", protocol);
            Err(std::io::Error::new(ErrorKind::Unsupported, format!{"Unsupported protocol '{}'", protocol}))
        }
    }
}

pub async fn start_test(server_ip: &str, server_port: u16, protocol: &str, test_duration: Duration, packets_per_second: u32, packet_size: usize, log_path: &str, output_rtt: bool, measure_owl: bool) -> std::io::Result<TestResult>
{
    let mut client = Client::new(server_ip, server_port, protocol, test_duration, packets_per_second, packet_size, log_path, output_rtt, measure_owl);
    let test_result = client.run_test().await.unwrap();

    Ok(test_result)
}
