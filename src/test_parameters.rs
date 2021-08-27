/*<copyright file="test_parameters.rs" company="Fraunhofer Institute for Manufacturing Engineering and Automation IPA">
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

use std::time::Duration;

#[derive(Clone)]
pub struct TestParameters {
    pub server_ip: String,
    pub server_port: u16,
    pub protocol: String,
    pub test_duration_valid: Duration,
    pub test_duration_total: Duration,
    pub warmup_duration: Duration,
    pub packets_per_second: u32,
    pub packet_size: usize,
    pub output_rtt: bool,
    pub measure_owl: bool
}
