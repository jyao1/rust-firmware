// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use bitfield::Bit;
use bitfield::BitRange;
use x86::msr;

pub const SIZE_4KB: u64 = 0x00001000u64;
pub const LOCAL_APIC_MODE_XAPIC: u64 = 0x1;
pub const LOCAL_APIC_MODE_X2APIC: u64 = 0x2;

pub fn local_apic_base_address_msr_supported() -> bool {
    let res = x86::cpuid::cpuid!(1u32);
    let res: u32 = res.eax.bit_range(11, 8);
    !(res == 0x4 || res == 0x05)
}

pub fn get_apic_mode() -> u64 {
    match local_apic_base_address_msr_supported() {
        false => LOCAL_APIC_MODE_XAPIC,
        true => {
            let base = unsafe { msr::rdmsr(msr::IA32_APIC_BASE) };

            //
            // [Bit 10] Enable x2APIC mode. Introduced at Display Family / Display
            // Model 06_1AH.
            //
            let ret: bool = base.bit(10);
            if ret {
                LOCAL_APIC_MODE_X2APIC
            } else {
                LOCAL_APIC_MODE_XAPIC
            }
        }
    }
}


pub fn set_apic_mode(mode: u64) {
    let current_mode = get_apic_mode();
    if current_mode == LOCAL_APIC_MODE_XAPIC && mode == LOCAL_APIC_MODE_X2APIC {
        unsafe {
            let mut base = msr::rdmsr(msr::IA32_APIC_BASE);
            base.set_bit(10, true);
            msr::wrmsr(msr::IA32_APIC_BASE, base);
        }
    }

    if current_mode == LOCAL_APIC_MODE_X2APIC && mode == LOCAL_APIC_MODE_XAPIC {
        //
        //  Transition from x2APIC mode to xAPIC mode is a two-step process:
        //    x2APIC -> Local APIC disabled -> xAPIC
        //
        unsafe {
            let mut base = msr::rdmsr(msr::IA32_APIC_BASE);
            base.set_bit(10, false);
            base.set_bit(11, false);
            msr::wrmsr(msr::IA32_APIC_BASE, base);
            base.set_bit(11, true);
            msr::wrmsr(msr::IA32_APIC_BASE, base);
        }
    }
}

/// Local APIC timer divide configurations.
///
/// Defines the APIC timer frequency as the processor frequency divided by a
/// specified value.

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerDivide {
    /// Divide by 2.
    Div2 = 0b0000,
    /// Divide by 4.
    Div4 = 0b0001,
    /// Divide by 8.
    Div8 = 0b0010,
    /// Divide by 16.
    Div16 = 0b0011,
    /// Divide by 32.
    Div32 = 0b1000,
    /// Divide by 64.
    Div64 = 0b1001,
    /// Divide by 128.
    Div128 = 0b1010,
    /// Divide by 256.
    Div256 = 0b1011,
}

/// Local APIC timer modes.

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerMode {
    /// Timer only fires once.
    OneShot = 0b00,
    /// Timer fires periodically.
    Periodic = 0b01,
    /// Timer fires at an absolute time.
    TscDeadline = 0b10,
}


pub fn initialize_apic_timer(
    divide_value: TimerDivide,
    init_count: u32,
    periodic_mode: TimerMode,
    vector: u8,
) {
    //
    // Ensure local APIC is in software-enabled state.
    //
    initialize_local_apic_software_enable(true);

    //
    // Program init-count register.
    //
    unsafe {
        msr::wrmsr(msr::IA32_X2APIC_INIT_COUNT, init_count as u64);
    }

    //
    // Enable APIC timer interrupt with specified timer mode.
    //
    unsafe {
        let _div_register = msr::rdmsr(msr::IA32_X2APIC_DIV_CONF);
        msr::wrmsr(msr::IA32_X2APIC_DIV_CONF, divide_value as u64);

        let mut lvt_timer_register = msr::rdmsr(msr::IA32_X2APIC_LVT_TIMER);

        lvt_timer_register.set_bit_range(18, 17, periodic_mode as u8);

        lvt_timer_register.set_bit_range(7, 0, vector);

        msr::wrmsr(msr::IA32_X2APIC_LVT_TIMER, lvt_timer_register);
    }
}


pub fn disable_apic_timer_interrupt() {
    unsafe {
        let mut lvt_timer_register = msr::rdmsr(msr::IA32_X2APIC_LVT_TIMER);
        lvt_timer_register.set_bit(16, true);

        msr::wrmsr(msr::IA32_X2APIC_LVT_TIMER, lvt_timer_register);
    }
}

fn initialize_local_apic_software_enable(b: bool) {
    let mut srv = unsafe { msr::rdmsr(msr::IA32_X2APIC_SIVR) };
    if b {
        if !srv.bit(8) {
            srv.set_bit(8, true);
            unsafe { msr::wrmsr(msr::IA32_X2APIC_SIVR, srv) }
        }
    } else if srv.bit(8) {
        srv.set_bit(8, false);
        unsafe { msr::wrmsr(msr::IA32_X2APIC_SIVR, srv) }
    }
}
