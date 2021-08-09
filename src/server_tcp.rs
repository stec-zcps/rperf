// <copyright file="server_tcp.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
// Copyright 2021 Fraunhofer Institute for Manufacturing Engineering and Automation IPA
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// </copyright>

pub mod server {
    use std::{thread, time, io};

    use tokio::net::{TcpListener};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::messages::InitMessage;
    use std::str::from_utf8;
    use thread_priority::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub async fn start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Configure thread
        // let core_ids = core_affinity::get_core_ids().unwrap();
        // core_affinity::set_for_current(core_ids[0]);
        assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
        // Open TCP socket
        let server_address = format!("{}:{}", "0.0.0.0", port);
        let listener = TcpListener::bind(server_address).await?;
        println!("Started TCP server on port '{}'", port);

        loop {
            let (mut socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                // Configure stream
                socket.set_nodelay(true).unwrap();

                // Wait for init message from client
                let mut buf_init = [0; 1500];
                let client_init_message: InitMessage;
                loop {
                    let n = match socket.read(&mut buf_init).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                    let client_init_message_json = from_utf8(&buf_init[..n]).unwrap();
                    let client_init_message_result = serde_json::from_str(client_init_message_json);

                    client_init_message = match client_init_message_result {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Problem deserializing init message from client '{}': {}", socket.peer_addr().unwrap(), e);
                            drop (socket);
                            return;
                        }
                    };
                    break;
                }
                println!("Client '{}' connected and wants to perform test with packet size '{} byte'", socket.peer_addr().unwrap(), client_init_message.packet_size);
                thread::sleep(time::Duration::from_millis(100));
                socket.write("OK".as_bytes()).await.unwrap();
                socket.flush().await.unwrap();

                let mut buf = vec![0; client_init_message.packet_size];
                loop {
                    // Wait for packets and ...
                    let _n = match socket.read_exact(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof  => {
                            println!("Client '{}' disconnected", socket.peer_addr().unwrap());
                            return;
                        }
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    // ... send new packet with index of received packet back
                    let mut payload = vec![1u8; 16];
                    payload[0..=7].copy_from_slice(&buf[0..=7]);
                    let current_system_time_unix_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let current_system_time_unix_epoch_ms = current_system_time_unix_epoch.as_secs() as f64
                        + current_system_time_unix_epoch.subsec_nanos() as f64 * 1e-9;
                    payload[8..=15].copy_from_slice(&current_system_time_unix_epoch_ms.to_be_bytes());

                    if let Err(e) = socket.write_all(&payload).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            });
        }
    }
}
