/*<copyright file="client.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
Copyright 2021 Fraunhofer Institute for Manufacturing Engineering and Automation IPA

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
</copyright>*/

pub mod client {
    use std::net::{TcpStream, UdpSocket};
    use std::{io, str, thread, process, time};

    use std::time::{Instant, SystemTime, Duration, UNIX_EPOCH};


    use std::sync::{Arc};

    use csv::Writer;

    use crate::test_parameters::TestParameters;

    use std::fs::File;
    use std::io::{Write, Read};
    use crate::test_result::TestResult;
    use std::str::from_utf8;
    use thread_priority::*;
    use crate::messages::InitMessage;


    use crate::packet::{SentPacket, ReceivedPacket};


    pub struct Client {
        pub test_parameters: TestParameters,
        server_address: String,
        sent_packets: Vec<SentPacket>,
        received_packets: Vec<ReceivedPacket>,
        message_interval: f64,
        expected_packet_count: u64,
        log_path: String
    }

    impl Client {
        pub fn new(server_ip: &str, server_port: u16, protocol: &str, test_duration: Duration, packets_per_second: u32, packet_size: usize, log_path: &str, output_rtt: bool, measure_owl: bool) -> Client {
            Client {
                test_parameters: TestParameters {
                    server_ip: server_ip.to_string(),
                    server_port: server_port.clone(),
                    protocol: protocol.to_string(),
                    time: test_duration,
                    packets_per_second,
                    packet_size,
                    output_rtt,
                    measure_owl
                },
                server_address: format!("{}:{}", &server_ip, &server_port),
                sent_packets: Vec::new(),
                received_packets: Vec::new(),
                message_interval: 1_f64 / packets_per_second.clone() as f64 * 1000_000_f64,
                expected_packet_count: test_duration.as_millis() as u64 / 1000_u64 * packets_per_second.clone() as u64,
                log_path: String::from(log_path)
            }
        }

        pub async fn run_test(&mut self){
            match &self.test_parameters.protocol.as_ref() {
                &"udp" => {
                    &self.run_udp_test().await.unwrap_or_else(|error| {
                       panic!("Problem starting test: {:?}", error);
                    });
                },
                &"tcp" => {
                    &self.run_tcp_test();
                },
                _ => println!("Unknown protocol"),
            }
        }

        pub fn generate_sent_packet(packet_index: &u64, packet_size: usize, sent_duration: Duration) -> (SentPacket, Vec<u8>) {
            let packet_index_bytes = packet_index.to_be_bytes();
            let mut payload = vec![1u8; packet_size];
            payload[0..=7].copy_from_slice(&packet_index_bytes);
            let current_system_time_unix_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let current_system_time_unix_epoch_ms = current_system_time_unix_epoch.as_secs() as f64
                + current_system_time_unix_epoch.subsec_nanos() as f64 * 1e-9;
            payload[8..=15].copy_from_slice(&current_system_time_unix_epoch_ms.to_be_bytes());

            let sent_packet = SentPacket {
                index: *packet_index,
                sent_duration: sent_duration,
                sent_timestamp: current_system_time_unix_epoch
            };

            return (sent_packet, payload);
        }

        pub fn generate_received_packet(buf: Vec<u8>, received_duration: Duration) -> ReceivedPacket {
            let mut packet_index_bytes = [0; 8];
            packet_index_bytes[..8].copy_from_slice(&buf[0..=7]);
            let received_packet_index = u64::from_be_bytes(packet_index_bytes);

            let mut timestamp_bytes = [0; 8];
            timestamp_bytes[..8].copy_from_slice(&buf[8..=15]);
            let server_timestamp_unix_epoch = Duration::from_secs_f64(f64::from_be_bytes(timestamp_bytes));

            let receive_timestamp_unix_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

            let received_packet = ReceivedPacket {
                index: received_packet_index,
                received_duration: received_duration,
                received_timestamp: receive_timestamp_unix_epoch,
                server_timestamp: server_timestamp_unix_epoch
            };

            return received_packet;
        }

        async fn run_udp_test(&mut self) -> std::io::Result<()> {
            let sender_socket = UdpSocket::bind("0.0.0.0:0")?;
            //let sender_socket = socket.clone();
            //socket.set_read_timeout(Some(time::Duration::from_secs(3)));
            sender_socket.connect(&self.server_address.clone())?;
            sender_socket.send("_client_".as_bytes())?;
            let receiver_socket = sender_socket.try_clone().unwrap();
            let mut buf = [0u8; 1500];
            match sender_socket.recv_from(&mut buf) {
                Ok((_number_of_bytes, _src_addr)) => {
                    println!("Server '{}' is reachable", &self.server_address);
                }
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                    println!("Server '{}' refused connection", &self.server_address);
                    process::exit(1);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                    println!("Server '{}' not reachable", &self.server_address);
                    process::exit(1);
                }
                Err(e) => panic!("Error communicating with server '{}': {:?}", &self.server_address, e)
            }

            let instant_sender_thread = Arc::new(Instant::now());
            let instant_receiver_thread = instant_sender_thread.clone();

            let _timer = howlong::HighResolutionTimer::new();

            println!("Starting test against server '{}'", &self.server_address);
            let expected_packet_count = self.expected_packet_count;
            let packet_size = self.test_parameters.packet_size;
            let message_interval = self.message_interval;
            let thread_send = thread::spawn(move || -> std::io::Result<Vec<SentPacket>> {

                // let core_ids = core_affinity::get_core_ids().unwrap();
                // core_affinity::set_for_current(core_ids[5]);
                assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());

                let mut sent_packets: Vec<SentPacket> = Vec::with_capacity((expected_packet_count + 10) as usize);
                let mut packet_index = 0_u64;
                let mut last_sent_time = SystemTime::now();

                while packet_index < expected_packet_count as u64 {

                    let (sent_packet, payload) = Client::generate_sent_packet(&packet_index, packet_size, instant_sender_thread.elapsed());
                    sent_packets.push(sent_packet);
                    sender_socket.send(&payload)?;

                    packet_index += 1;

                    while last_sent_time.elapsed().unwrap().as_micros() < message_interval as u128 {
                    }
                    last_sent_time = SystemTime::now();
                }

                Ok(sent_packets)
            });

            let test_duration = self.test_parameters.time;
            let thread_receive = thread::spawn(move || -> std::io::Result<Vec<ReceivedPacket>> {
                // Configure thread
                // let core_ids = core_affinity::get_core_ids().unwrap();
                // core_affinity::set_for_current(core_ids[5]);
                assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());

                // Configure socket
                //socket_clone.set_read_timeout(Some(time::Duration::from_secs(1)));

                let mut received_packets: Vec<ReceivedPacket> = Vec::with_capacity((expected_packet_count + 10) as usize);

                // Open UDP socket
                //let server_address = format!("{}:{}", "0.0.0.0", 5556);
                //let receiver_socket = UdpSocket::bind(server_address).unwrap();
                receiver_socket.set_read_timeout(Some(time::Duration::from_secs(3)))?;
                //println!("Started UDP server on port '{}'", 5556);

                'outer: while instant_receiver_thread.elapsed() < test_duration + time::Duration::from_millis(1000) {
                    let mut buf = [0u8; 1500];

                    let (_num_bytes_read, _) = loop {
                        match receiver_socket.recv_from(&mut buf) {
                            Ok(n) => break n,
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                                break 'outer;
                            }
                            Err(e) => panic!("encountered IO error: {}", e),
                        }
                    };

                    let received_packet = Client::generate_received_packet(buf.to_vec(), instant_receiver_thread.elapsed());
                    received_packets.push(received_packet);
                }

                println!("Receive thread finished");

                Ok(received_packets)
            });

            self.sent_packets = thread_send.join().unwrap().unwrap();
            self.received_packets = thread_receive.join().unwrap().unwrap();

            self.generate_result()?;

            return Ok(());
        }

        pub fn run_tcp_test(&mut self) -> std::io::Result<()> {
            match TcpStream::connect(&self.server_address.clone()) {
                Ok(mut stream) => {
                    // Send init message and wait for response
                    let init_message = InitMessage {
                        packet_size: self.test_parameters.packet_size
                    };
                    let init_message_json = serde_json::to_string(&init_message).unwrap();
                    stream.write(init_message_json.as_bytes())?;
                    let mut buf = [0; 1500];
                    loop {
                        let n = match stream.read(&mut buf) {
                            Ok(n) if n == 0 => continue,
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("failed to read from socket; err = {:?}", e);
                                continue;
                            }
                        };
                        let server_response = from_utf8(&buf[..n]).unwrap();
                        if server_response.eq("OK")
                        {
                            println!("Successfully connected to server '{}'", &self.server_address);
                            break;
                        }
                        else
                        {
                            panic!("Invalid server response: {}", server_response);
                        }
                    }

                    println!("Starting test against server '{}'", &self.server_address);
                    let mut stream_clone = stream.try_clone().unwrap();
                    let instant_sender_thread = Arc::new(Instant::now());
                    let instant_receiver_thread = instant_sender_thread.clone();

                    let expected_packet_count = self.expected_packet_count;
                    let packet_size = self.test_parameters.packet_size;
                    let message_interval = self.message_interval;
                    let thread_send = thread::spawn(move || {
                        // Configure thread
                        assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
                        // Configure stream
                        stream.set_nodelay(true).unwrap();

                        let mut sent_packets: Vec<SentPacket> = Vec::with_capacity((expected_packet_count + 10) as usize);
                        let mut packet_index = 0_u64;
                        let mut last_sent_time = SystemTime::now();
                        while packet_index < expected_packet_count as u64 {

                            let (sent_packet, payload) = Client::generate_sent_packet(&packet_index, packet_size, instant_sender_thread.elapsed());

                            sent_packets.push(sent_packet);
                            stream.write(&payload).unwrap();
                            stream.flush().unwrap();

                            packet_index += 1;

                            while last_sent_time.elapsed().unwrap().as_micros() < message_interval as u128 {
                            }
                            last_sent_time = SystemTime::now();
                        }

                        sent_packets
                    });

                    let test_duration = self.test_parameters.time;
                    let thread_receive = thread::spawn(move || -> Result<Vec<ReceivedPacket>, std::io::Error> {
                        // Configure thread
                        assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
                        // Configure stream
                        stream_clone.set_read_timeout(Some(time::Duration::from_secs(10)))?;

                        let mut received_packets: Vec<ReceivedPacket> = Vec::with_capacity((expected_packet_count + 10) as usize);
                        'outer: while instant_receiver_thread.elapsed() < test_duration + time::Duration::from_millis(1000) {

                            let mut buf = [0u8; 16];
                            match stream_clone.read_exact(&mut buf) {
                                Ok(_) => {
                                    let received_packet = Client::generate_received_packet(buf.to_vec(), instant_receiver_thread.elapsed());
                                    received_packets.push(received_packet);
                                },
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                                    break 'outer;
                                }
                                Err(e) => {
                                    println!("Failed to receive data: {}", e);
                                }
                            }
                        }

                        println!("Receive thread finished");
                        return Ok(received_packets);


                    });

                    self.sent_packets = thread_send.join().unwrap();
                    self.received_packets = thread_receive.join().unwrap().unwrap();

                    self.generate_result()?;
                    println!("Terminated.");

                    return Ok(());
                },
                Err(e) => {
                    println!("Failed to connect: {}", e);
                }
            }

            return Ok(());
        }

        fn generate_result(&mut self) -> std::io::Result<()> {
            let test_result = TestResult::from_tx_rx_times(self.test_parameters.clone(), &self.sent_packets, &self.received_packets, self.test_parameters.output_rtt);

            println!("Sent Duration [ms]: {:.3}", test_result.sent_duration_millis);
            println!("Sent Packets: {}", test_result.sent_packets_count);
            println!("Received Packets: {}", test_result.received_packets_count);
            println!("Lost Packets: {}", test_result.lost_packets_count);
            if self.test_parameters.output_rtt
            {
                println!("Average Round Trip Time: {}", test_result.average_latency());
            } else {
                println!("Average Latency: {}", test_result.average_latency());
            }

            if !&self.log_path.is_empty() {
                let mut file = File::create(&self.log_path)?;
                file.write_all(format!("Test Parameters: --ip {} -p {} --time {} --mps {}, --size {}\n\
                                    Test Results: Sent Duration [s]: {:.3} | Sent Packets: {}, Received Packets: {}, Lost Packets: {}, Average Latency [ms]: {}\n",
                                       test_result.test_parameters.server_ip,
                                       test_result.test_parameters.server_port,
                                       test_result.test_parameters.time.as_secs(),
                                       test_result.test_parameters.packets_per_second,
                                       test_result.test_parameters.packet_size,
                                       test_result.sent_duration_millis,
                                       test_result.sent_packets_count,
                                       test_result.received_packets_count,
                                       test_result.lost_packets_count,
                                       test_result.average_latency()).as_bytes())?;

                let mut csv_writer = Writer::from_writer(file);

                if self.test_parameters.output_rtt {
                    csv_writer.write_record(&["Packet", "TxTime[s]", "RxTime[s]", "RoundTripTime[ms]", "OneWayLatency_ClientToServer[ms]", "OneWayLatency_ServerToClient[ms]"])?;
                }
                else {
                    csv_writer.write_record(&["Packet", "TxTime[s]", "RxTime[s]", "Latency[ms]", "OneWayLatency_ClientToServer[ms]", "OneWayLatency_ServerToClient[ms]"])?;
                }
                for packet_result in test_result.packet_results {
                    csv_writer.write_record(&[packet_result.index.to_string(),
                        format!("{:.20}", packet_result.tx_time),
                        format!("{:.20}", packet_result.rx_time),
                        format!("{:.9}", packet_result.latency),
                        format!("{:.9}", packet_result.latency_client_to_server),
                        format!("{:.9}", packet_result.latency_server_to_client)])?;
                }
                csv_writer.flush()?;

                println!("Test results logged in '{}'", &self.log_path);
            }

            println!("Test finished");

            Ok(())
        }
    }
}
