// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub const SIZE_16MB: u64 = 0x01000000u64;

pub fn cmos_read8(index: u8) -> u8 {
    unsafe {
        x86::io::outb(0x70, index);
        x86::io::inb(0x71)
    }
}

pub fn cmos_write8(index: u8, value: u8) -> u8 {
    unsafe {
        x86::io::outb(0x70, index);
        x86::io::outb(0x71, value);
    }
    0
}

///
/// CMOS 0x34/0x35 specifies the system memory above 16 MB.
///
/// CMOS(0x35) is the high byte
/// CMOS(0x34) is the low byte
/// The size is specified in 64kb chunks
/// Since this is memory above 16MB, the 16MB must be added
/// into the calculation to get the total memory size.
///
pub fn get_system_memory_size_below4_gb() -> u64 {

    let cmos0x34 = cmos_read8(0x34u8);
    let cmos0x35 = cmos_read8(0x35u8);

    (((cmos0x35 as u64) << 8 + (cmos0x34 as u64)) << 16) + SIZE_16MB
}
