// <copyright file="server_udp.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
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
    use std::net::UdpSocket;
    use std::{io, str};
    use thread_priority::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn start(port: u16) -> std::io::Result<()> {
        // Configure thread
        //let core_ids = core_affinity::get_core_ids().unwrap();
        //core_affinity::set_for_current(core_ids[0]);
        //assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());

        // Open UDP socket
        let server_address = format!("{}:{}", "0.0.0.0", port);
        let socket = UdpSocket::bind(server_address)?;
        println!("Started UDP server on port '{}'", port);

        // Wait for packets
        loop {
            let mut buf = [0u8; 1500];
            let (amt, src) = socket.recv_from(&mut buf)?;

            let mut payload = vec![1u8; 16];
            payload[0..=7].copy_from_slice(&buf[0..=7]);
            let current_system_time_unix_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let current_system_time_unix_epoch_ms = current_system_time_unix_epoch.as_secs() as f64
                + current_system_time_unix_epoch.subsec_nanos() as f64 * 1e-9;
            payload[8..=15].copy_from_slice(&current_system_time_unix_epoch_ms.to_be_bytes());

            socket.send_to(&payload, &src)?;
        }
    }
}
