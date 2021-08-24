[![Build Status](https://app.travis-ci.com/stec-zcps/rperf.svg?branch=main)](https://app.travis-ci.com/github/stec-zcps/rperf)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fstec-zcps%2Frperf.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fstec-zcps%2Frperf?ref=badge_shield)

# Introduction
Rperf is a network latency measurement tool written in Rust. It aims to analyze the network latency of packets with 
different test parameters (e.g. network protocol, packet size, packet rate). The following features and parameters are 
currently supported:
* Measure round trip time of packets via ping pong between client and server
* Measure one way latencies of packets between client and server using system timestamps 
  (system clocks of client and server need to be synchronized!)
* UDP or TCP for packet transmission between client and server
* Configurable packet rate per second
* Configurable payload size of packets (min. 16 bytes)
* Create symmetric or asymmetric network load by using minimal pong packets or packet mirroring
* Output test results CSV file

# Build
## Executable
Prerequisites:
* [Install Rust](https://www.rust-lang.org/tools/install)

Use the following command to build the tool. After build executable 'rperf' can be found in target/release directory.
```bash
cargo build --release
```
## Docker Image
```bash
docker build -t stecipa/rperf:1.0.0-SNAPSHOT .
```

# Use
Rperf Docker Image is available on [Docker Hub](https://hub.docker.com/repository/docker/stecipa/rperf).

Rperf can be started in server or client mode. Use the following command to display the help text and the available parameters:
```bash
rperf help 
```
## Server
Usage:
```bash
rperf server --port <Port> --protocol <Protocol>
```
Example:
```bash
rperf server --port 5555 --protocol udp
```
## Client
Usage:
```bash
rperf client [FLAGS] [OPTIONS] --ip <IP> --port <Port> --time <time> --mps <time> --size <size> --protocol <Protocol>
```
Example:
```bash
rperf client --ip 127.0.0.1 -p 5555 --time 2 --mps 1000 --size 64 --protocol udp --log result.csv --owl --rtt
```




## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fstec-zcps%2Frperf.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fstec-zcps%2Frperf?ref=badge_large)