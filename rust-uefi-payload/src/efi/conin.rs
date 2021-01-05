// Copyright © 2019 Intel Corporation
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

// Inspired by https://github.com/phil-opp/blog_os/blob/post-03/src/vga_buffer.rs
// from Philipp Oppermann

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use cpuio::Port;
use core::ffi::c_void;

const LSR_TXRDY: u8 = 0x20;
const LSR_RXDA : u8 = 0x01;
const LSR_OFFSET: u16 = 0x05;

pub struct ConIn {
    port: Port<u8>,
    lsr_port: Port<u8>,
}

impl ConIn {
    pub fn read_byte(&mut self) -> u8 {

        let data = self.lsr_port.read();
        if (data & LSR_RXDA) == 0 {
          return 0;
        }
        let byte = self.port.read();
        byte
    }

    pub fn new() -> ConIn {
        ConIn {
            port: unsafe { Port::new(0x3f8) },
            lsr_port: unsafe { Port::new(0x3f8 + LSR_OFFSET) },
        }
    }
}
