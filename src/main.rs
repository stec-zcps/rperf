/*<copyright file="main.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
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

mod client;
mod server_udp;
mod test_parameters;
mod packet_result;
mod test_result;
mod server_tcp;
mod messages;
mod packet;

use clap::{App, Arg};
use std::time::Duration;

use crate::client::client::Client;
use thread_priority::{set_current_thread_priority, ThreadPriority};
use affinity::get_core_num;
use std::process;

#[tokio::main]
async fn main() {

    let matches = App::new("Rperf")
        .version("1.0.0")
        .author("Matthias Schneider <matthias.schneider@ipa.fraunhofer.de")
        .about("Latency measurement tool implemented in Rust.")
        .subcommand(App::new("server")
            .about("Run a Rperf server")
            .version("1.0.0")
            .author("Matthias Schneider <matthias.schneider@ipa.fraunhofer.de")
            .arg(Arg::new("port")
                .short('p')
                .long("port")
                .value_name("Port")
                .about("Port of the server")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("protocol")
                .long("protocol")
                .value_name("Protocol")
                .about("Protocol of the server [tpc|udp]")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("sym-load")
                .long("sym-load")
                .value_name("sym-load")
                .about("Creates symmetric network load between client and server using ping packet size for pong packets. If this flag is not set pong packets have minimal packet size (16 bytes).")
                .required(false)
                .takes_value(false))
        )
        .subcommand(App::new("client")
            .about("Execute latency test as client against a Rperf server")
            .version("1.0.0")
            .author("Matthias Schneider <matthias.schneider@ipa.fraunhofer.de")
            .arg(Arg::new("ip")
                .short('i')
                .long("ip")
                .value_name("IP")
                .about("IP of Rperf server which will be used for testing")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("port")
                .short('p')
                .long("port")
                .value_name("Port")
                .about("Port of Rperf server which will be used for testing")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("time")
                .short('t')
                .long("time")
                .value_name("time")
                .about("Duration of test [seconds]")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("mps")
                .long("mps")
                .value_name("time")
                .about("Messages send per second")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("size")
                .long("size")
                .value_name("size")
                .about("Payload size of messages [bytes (min. 16)]")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("protocol")
                .long("protocol")
                .value_name("Protocol")
                .about("Protocol used to communicate with Rperf server [tcp|udp]")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("log")
                .long("log")
                .value_name("Log")
                .about("Path to log test results")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("rtt")
                .long("rtt")
                .value_name("rtt")
                .about("Output result as round trip time instead of latency (latency = round trip time /2 if --owl flag is not used)")
                .required(false)
                .takes_value(false))
            .arg(Arg::new("owl")
                .long("owl")
                .value_name("owl")
                .about("Measure one way latencies using timestamps of system clocks (client and server clock needs to be synchronized!)")
                .required(false)
                .takes_value(false))
        )
        .get_matches();

    let _cores: Vec<usize> = (0..get_core_num()).collect();
    println!("Core nums : {}", get_core_num());
    assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());

    if let Some(ref matches) = matches.subcommand_matches("server") {
        println!("Server Mode");

        let protocol = matches.value_of("protocol").unwrap();
        let port = matches.value_of_t("port").unwrap();
        let symmetric_network_load = matches.is_present("sym-load");

        match protocol {
            "udp" => {
                server_udp::server::start(port, symmetric_network_load).unwrap_or_else(|error| {
                    panic!("Problem running test: {:?}", error);
                });
            },
            "tcp" => {
                server_tcp::server::start(port, symmetric_network_load).await.unwrap_or_else(|error| {
                    panic!("Problem running test: {:?}", error);
                });
            },
            _ => println!("Unknown protocol"),
        }
    }
    else if let Some(ref matches) = matches.subcommand_matches("client") {
        println!("Client Mode");

        let ip = matches.value_of("ip").unwrap();
        let port: u16 = matches.value_of_t("port").unwrap();
        let protocol = matches.value_of("protocol").unwrap();
        let time = matches.value_of_t("time").unwrap();
        let mps = matches.value_of_t("mps").unwrap();
        let size = matches.value_of_t("size").unwrap();
        let log_path = matches.value_of("log").unwrap_or_default();

        let output_rtt = matches.is_present("rtt");
        let measure_owl = matches.is_present("owl");

        if let Some(ip) = matches.value_of("ip") {
            println!("IP: {}", ip);
        }

        if size < 16
        {
            eprintln!("Packet size must be at least 16 bytes!");
            process::exit(1);
        }

        let mut client = Client::new(ip, port, protocol, Duration::from_secs(time), mps, size, log_path, output_rtt, measure_owl);
        client.run_test().await;
    }
}
