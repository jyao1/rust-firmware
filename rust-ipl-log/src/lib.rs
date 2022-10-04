// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![cfg_attr(not(test), no_std)]

use core::fmt::{self, Write};
use x86_64::instructions::port::Port;

const DEFAULT_LOG_LEVEL: usize = LOG_LEVEL_INFO;

pub const LOG_LEVEL_VERBOSE: usize = 1000;
pub const LOG_LEVEL_INFO: usize = 100;
pub const LOG_LEVEL_WARN: usize = 10;
pub const LOG_LEVEL_ERROR: usize = 1;
pub const LOG_LEVEL_NONE: usize = 0;

pub const LOG_MASK_COMMON: u64 = 0x1;
// Core - Boot Service (BIT1 ~ BIT15)
pub const LOG_MASK_PROTOCOL: u64 = 0x2;
pub const LOG_MASK_MEMORY: u64 = 0x4;
pub const LOG_MASK_EVENT: u64 = 0x8;
pub const LOG_MASK_IMAGE: u64 = 0x10;
// Core - Runtime Service (BIT16 ~ BIT 23)
pub const LOG_MASK_VARIABLE: u64 = 0x10000;
// Core - Console (BIT24 ~ BIT 31)
pub const LOG_MASK_CONOUT: u64 = 0x1000000;
pub const LOG_MASK_CONIN: u64 = 0x2000000;
// Protocol - (BIT32 ~ BIT63)
pub const LOG_MASK_BLOCK_IO: u64 = 0x100000000;
pub const LOG_MASK_FILE_SYSTEM: u64 = 0x200000000;
// All
pub const LOG_MASK_ALL: u64 = 0xFFFFFFFFFFFFFFFF;

pub struct Logger {
    port: Port<u8>,
    level: usize,
    mask: u64,
}

impl Logger {
    fn port_write(&mut self, byte: u8) {
        unsafe {
            self.port.write(byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.port_write(b'\r')
        }
        self.port_write(byte)
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }

    pub fn get_level(&mut self) -> usize {
        self.level
    }
    pub fn set_level(&mut self, level: usize) {
        self.level = level;
    }

    pub fn get_mask(&mut self) -> u64 {
        self.mask
    }
    pub fn set_mask(&mut self, mask: u64) {
        self.mask = mask;
    }
}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn write_log(level: usize, mask: u64, args: fmt::Arguments) {
    let mut logger = Logger {
        port: Port::new(0x3f8),
        level: DEFAULT_LOG_LEVEL,
        mask: LOG_MASK_ALL,
    };

    if level > logger.get_level() {
        return;
    }
    if (mask & logger.get_mask()) == 0 {
        return;
    }
    logger.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        log::write_log(log::LOG_LEVEL_INFO, log::LOG_MASK_COMMON, format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => (
        log::write_log(log::LOG_LEVEL_VERBOSE, log::LOG_MASK_COMMON, format_args!($($arg)*))
    );
}
