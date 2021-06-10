// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[macro_use]
use crate::logger;

use crate::pcd;
use r_uefi_pi::fv;
use crate::pci;

use r_efi::efi;
use core::panic::PanicInfo;
use core::ffi::c_void;
use core::convert::TryInto;
use core::slice;

use x86::bits64::paging;
use x86::msr;
use bitfield::BitRange;
use bitfield::Bit;

pub const SIZE_4KB  :u64 = 0x00001000u64;
pub const SIZE_1MB  :u64 = 0x00100000u64;
pub const SIZE_2MB  :u64 = 0x00200000u64;
pub const SIZE_16MB :u64 = 0x01000000u64;

pub const LOCAL_APIC_MODE_XAPIC :u64 = 0x1;
pub const LOCAL_APIC_MODE_X2APIC :u64 = 0x2;

fn cmos_read8(index: u8) -> u8
{
    let mut res: u8 = 0;
    unsafe {
        x86::io::outb(0x70, index);
        res = x86::io::inb(0x71);
    }
    res
}

fn cmos_write8(index: u8, value: u8) -> u8
{
    let mut res: u8 = 0;
    unsafe {
        x86::io::outb(0x70, index);
        x86::io::outb(0x71, value);
    }
    res
}

#[allow(non_snake_case)]
pub fn GetSystemMemorySizeBelow4Gb() -> u64 {
    let mut cmos0x34: u8 = 0u8;
    let mut cmos0x35: u8 = 0u8;

    cmos0x34 = cmos_read8(0x34u8);
    cmos0x35 = cmos_read8(0x35u8);
    let mut res: u64 = 0;
    res = (((cmos0x35 as u64) << 8 + (cmos0x34 as u64)) << 16) + SIZE_16MB;
    res
}

#[allow(non_snake_case)]
pub fn EfiSizeToPage(size: u64) -> u64
{
    (size + SIZE_4KB - 1) / SIZE_4KB
}

#[allow(non_snake_case)]
pub fn EfiPageToSize(page: u64) -> u64
{
    page * SIZE_4KB
}

/// flag  ture align to low address else high address
#[allow(non_snake_case)]
fn AlignValue(value: u64, align: u64, flag: bool) -> u64
{
    if flag {
        value & ((!(align - 1)) as u64)
    } else {
        value - (value & (align - 1)) as u64 + align
    }
}

#[allow(non_snake_case)]
pub fn FindAndReportEntryPoint(firmwareVolumePtr: * const fv::FirmwareVolumeHeader, loaded_buffer: &mut [u8]) -> (u64, u64, u64)
{
    let firmware_buffer = unsafe { core::slice::from_raw_parts(firmwareVolumePtr as *const u8, pcd::pcd_get_PcdOvmfDxeMemFvSize() as usize) };
    let image = uefi_pi::fv_lib::get_image_from_fv(firmware_buffer, fv::FV_FILETYPE_DXE_CORE, fv::SECTION_PE32).unwrap();
    if elf_loader::elf::is_elf(image) {
        log!("payload is elf image\n");
        elf_loader::elf::relocate_elf(image, loaded_buffer)
    } else if pe_loader::pe::is_pe(image) {
        log!("payload is pe image\n");
        pe_loader::pe::relocate_pe_mem(image, loaded_buffer)
    } else {
        panic!("format not support")
    }

}

#[allow(non_snake_case)]
pub fn PciExBarInitialization()
{
    let pci_exbar_base = pcd::pcd_get_PcdPciExpressBaseAddress();

    //
    // Clear the PCIEXBAREN bit first, before programming the high register.
    //
    pci::PciCf8Write32(0, 0, 0, 0x60, 0);

    //
    // Program the high register. Then program the low register, setting the
    // MMCONFIG area size and enabling decoding at once.
    //
    log!("pci_exbar_base {:x}\n", pci_exbar_base);
    log!("pci_exbar_base {:x}, {:x}\n", (pci_exbar_base >> 32) as u32, (pci_exbar_base << 32 >> 32 | 0x1) as u32);
    pci::PciCf8Write32(0, 0, 0, 0x64, (pci_exbar_base >> 32) as u32);
    pci::PciCf8Write32(0, 0, 0, 0x60, (pci_exbar_base << 32 >> 32 | 0x1) as u32);
}

#[allow(non_snake_case)]
pub fn InitPci()
{
    pci::PciCf8Write32(0, 3, 0, 0x14, 0xC1085000);
    pci::PciCf8Write32(0, 3, 0, 0x20, 0xC200000C);
    pci::PciCf8Write32(0, 3, 0, 0x24, 0x00000008);
    pci::PciCf8Write8(0, 3, 0, 0x4, 0x07);
}

#[allow(non_snake_case)]
pub fn VirtIoBlk()
{
    let base: usize = 0x8C2000000usize;
    use core::intrinsics::volatile_store;

    log!("VIRTIO_STATUS_RESET\n");
    unsafe{volatile_store((base + 0x14usize) as *mut u32, 0u32);}
    log!("VIRTIO_STATUS_ACKNOWLEDGE\n");
    unsafe{volatile_store((base + 0x14usize) as *mut u32, 1u32);}
    log!("VIRTIO_STATUS_DRIVER\n");
    unsafe{volatile_store((base + 0x14usize) as *mut u32, 2u32);}
}

#[allow(non_snake_case)]
pub fn CpuGetMemorySpaceSize() -> u8
{
    let res = x86::cpuid::cpuid!(0x80000000u32);
    if res.eax > 0x80000008u32 {
        let res = x86::cpuid::cpuid!(0x80000008u32);
        let sizeofmemoryspace = (res.eax & 0xffu32) as u8;
        sizeofmemoryspace
    }else {
        0u8
    }
}


#[allow(non_snake_case)]
pub fn LocalApicBaseAddressMsrSupported() -> bool
{
    let res = x86::cpuid::cpuid!(1u32);
    let res: u32 = res.eax.bit_range(11, 8);
    if res == 0x4 || res == 0x05 {
        false
    } else {
        true
    }
}

#[allow(non_snake_case)]
pub fn GetApicMode() -> u64
{
    use x86::msr;
    match LocalApicBaseAddressMsrSupported() {
        false => LOCAL_APIC_MODE_XAPIC,
        true => {
            let base = unsafe{msr::rdmsr(msr::IA32_APIC_BASE)};

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

#[allow(non_snake_case)]
pub fn SetApicMode(mode: u64) {
    let CurrentMode = GetApicMode();
    if CurrentMode == LOCAL_APIC_MODE_XAPIC && mode == LOCAL_APIC_MODE_X2APIC {
        unsafe {
            let mut base = msr::rdmsr(msr::IA32_APIC_BASE);
            base.set_bit(10, true);
            msr::wrmsr(msr::IA32_APIC_BASE, base);
        }

    }

    if CurrentMode == LOCAL_APIC_MODE_X2APIC && mode == LOCAL_APIC_MODE_XAPIC {
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
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
pub fn InitializeApicTimer(DivideValue: TimerDivide, InitCount: u32, PeriodicMode: TimerMode, Vector: u8)
{
    //
    // Ensure local APIC is in software-enabled state.
    //
    InitializeLocalApicSoftwareEnable(true);

    //
    // Program init-count register.
    //
    unsafe {
        msr::wrmsr(msr::IA32_X2APIC_INIT_COUNT, InitCount as u64);
    }

    //
    // Enable APIC timer interrupt with specified timer mode.
    //
    unsafe {
        let mut div_register = msr::rdmsr(msr::IA32_X2APIC_DIV_CONF);
        msr::wrmsr(msr::IA32_X2APIC_DIV_CONF, DivideValue as u64);

        let mut lvt_timer_register = msr::rdmsr(msr::IA32_X2APIC_LVT_TIMER);

        lvt_timer_register.set_bit_range(18, 17, PeriodicMode as u8);

        lvt_timer_register.set_bit_range(7, 0, Vector);

        msr::wrmsr(msr::IA32_X2APIC_LVT_TIMER, lvt_timer_register);
    }

}

#[allow(non_snake_case)]
pub fn DisableApicTimerInterrupt()
{
    unsafe {
        let mut lvt_timer_register = msr::rdmsr(msr::IA32_X2APIC_LVT_TIMER);
        lvt_timer_register.set_bit(16, true);

        msr::wrmsr(msr::IA32_X2APIC_LVT_TIMER, lvt_timer_register);
    }
}

#[allow(non_snake_case)]
fn InitializeLocalApicSoftwareEnable(b: bool)
{
    let mut srv = unsafe {msr::rdmsr(msr::IA32_X2APIC_SIVR)};
    if b {
        if srv.bit(8) == false {
            srv.set_bit(8, true);
            unsafe { msr::wrmsr(msr::IA32_X2APIC_SIVR, srv)}
        }
    } else {
        if srv.bit(8) == true {
            srv.set_bit(8, false);
            unsafe { msr::wrmsr(msr::IA32_X2APIC_SIVR, srv)}
        }
    }
}
