/*<copyright file="test_result.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
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

use std::collections::{LinkedList, HashMap};
use crate::packet_result::PacketResult;
use crate::test_parameters::TestParameters;
use crate::packet::{SentPacket, ReceivedPacket};

use std::ops::Sub;

#[derive(Clone)]
pub struct TestResult {
    pub test_parameters: TestParameters,
    pub packet_results: LinkedList<PacketResult>,
    pub sent_duration_millis: f64,
    pub sent_packets_count: u64,
    pub received_packets_count: u64,
    pub lost_packets_count: u64
}

impl TestResult {
    pub fn from_tx_rx_times(test_parameters: TestParameters, sent_packets: &Vec<SentPacket>, received_packets: &Vec<ReceivedPacket>, output_rtt: bool) -> TestResult {

        let sent_packet_count = sent_packets.len();
        let received_packets_count = received_packets.len();
        let mut lost_packet_count = 0;

        let mut packet_results: LinkedList<PacketResult> = LinkedList::new();

        let mut receive_packets_map: HashMap<u64, &ReceivedPacket> = HashMap::with_capacity(received_packets.len());
        for received_packet in received_packets
        {
            receive_packets_map.insert(received_packet.index, received_packet);
        }

        for sent_packet in sent_packets {
            if receive_packets_map.contains_key(&sent_packet.index)
            {
                let received_packet = *receive_packets_map.get(&sent_packet.index).unwrap();
                let received_time = received_packet.received_duration;



                let mut latency_ms: f64 = -1_f64;
                let mut one_way_latency_client_to_server_ms: f64 = -1_f64;
                let mut one_way_latency_server_to_client_ms: f64 = -1_f64;
                if test_parameters.measure_owl {
                    let one_way_latency_client_to_server = received_packet.server_timestamp.checked_sub(sent_packet.sent_timestamp);
                    let one_way_latency_server_to_client = received_packet.received_timestamp.checked_sub(received_packet.server_timestamp);
                    if one_way_latency_client_to_server.is_some() && one_way_latency_server_to_client.is_some()
                    {
                        one_way_latency_client_to_server_ms = (one_way_latency_client_to_server.unwrap().as_secs() as f64 + one_way_latency_client_to_server.unwrap().subsec_nanos() as f64 * 1e-9) * 1000_f64;
                        one_way_latency_server_to_client_ms = (one_way_latency_server_to_client.unwrap().as_secs() as f64 + one_way_latency_server_to_client.unwrap().subsec_nanos() as f64 * 1e-9) * 1000_f64;
                        latency_ms = one_way_latency_client_to_server_ms + one_way_latency_server_to_client_ms;
                    }
                    else {
                        println!("Client Sent Timestamp: {}", (sent_packet.sent_timestamp.as_secs() as f64 + sent_packet.sent_timestamp.subsec_nanos() as f64 * 1e-9) * 1000_f64);
                        println!("Server Timestamp: {}", (received_packet.server_timestamp.as_secs() as f64 + received_packet.server_timestamp.subsec_nanos() as f64 * 1e-9) * 1000_f64);
                        println!("Client Receive Timestamp: {}", (received_packet.received_timestamp.as_secs() as f64 + received_packet.received_timestamp.subsec_nanos() as f64 * 1e-9) * 1000_f64);
                    }
                }
                else {
                    let round_trip_time = received_time.sub(sent_packet.sent_duration);
                    let round_trip_time_ms = (round_trip_time.as_secs() as f64 + round_trip_time.subsec_nanos() as f64 * 1e-9) * 1000_f64;
                    latency_ms = round_trip_time_ms;
                }

                if !output_rtt
                {
                    latency_ms = latency_ms / 2_f64;
                }

                let packet_result = PacketResult {
                    index: sent_packet.index,
                    rx_time: sent_packet.sent_duration.as_secs() as f64 + sent_packet.sent_duration.subsec_nanos() as f64 * 1e-9,
                    tx_time: received_time.as_secs() as f64 + received_time.subsec_nanos() as f64 * 1e-9,
                    latency: latency_ms,
                    latency_client_to_server: one_way_latency_client_to_server_ms,
                    latency_server_to_client: one_way_latency_server_to_client_ms
                };
                packet_results.push_back(packet_result);
            }
            else {
                lost_packet_count += 1;
            }
        }

        let mut vec: Vec<_> = packet_results.into_iter().collect();
        vec.sort_by(|a, b| a.index.cmp(&b.index));
        let packet_results: LinkedList<_> = vec.into_iter().collect();

        let sent_duration = packet_results.back().unwrap().tx_time.sub(packet_results.front().unwrap().tx_time);

        let test_result = TestResult {
            test_parameters: test_parameters,
            packet_results: packet_results,
            sent_duration_millis: sent_duration,
            sent_packets_count: sent_packet_count as u64,
            received_packets_count: received_packets_count as u64,
            lost_packets_count: lost_packet_count
        };

        return test_result;
    }

    pub fn average_latency(&self) -> f64 {
        let mut average_latency = 0_f64;
        for packet_result in &self.packet_results {
            average_latency += packet_result.latency;
        }
        average_latency = average_latency / self.packet_results.len() as f64;

        return average_latency;
    }

}
