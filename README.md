# Introduction

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


