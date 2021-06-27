// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub const SIZE_4KB: u64 = 0x00001000u64;
pub const SIZE_1MB: u64 = 0x00100000u64;
pub const SIZE_2MB: u64 = 0x00200000u64;
pub const SIZE_16MB: u64 = 0x01000000u64;

fn cmos_read8(index: u8) -> u8 {
    unsafe {
        x86::io::outb(0x70, index);
        x86::io::inb(0x71)
    }
}

fn cmos_write8(index: u8, value: u8) -> u8 {
    unsafe {
        x86::io::outb(0x70, index);
        x86::io::outb(0x71, value);
    }
    0
}

pub fn get_system_memory_size_below4_gb() -> u64 {
    let cmos0x34 = cmos_read8(0x34u8);
    let cmos0x35 = cmos_read8(0x35u8);

    (((cmos0x35 as u64) << 8 + (cmos0x34 as u64)) << 16) + SIZE_16MB
}

pub fn cpu_get_memory_space_size() -> u8 {
    let res = x86::cpuid::cpuid!(0x80000000u32);
    if res.eax > 0x80000008u32 {
        let res = x86::cpuid::cpuid!(0x80000008u32);
        let sizeofmemoryspace = (res.eax & 0xffu32) as u8;
        sizeofmemoryspace
    } else {
        0u8
    }
}
