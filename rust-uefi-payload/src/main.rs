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

#![allow(unused)]

#![feature(asm)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

#[macro_use]
mod logger;

#[macro_use]
mod common;

use core::panic::PanicInfo;

use core::ffi::c_void;

use cpuio::Port;

mod block;
mod bzimage;
mod efi;
mod pi;
mod fat;
mod loader;
mod mem;
mod mmio;
mod part;
mod pci;
mod pe;
mod virtio;
mod r_efi_ext;

#[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &PanicInfo) -> ! {
    log!("panic ... {:?}\n", _info);
    loop {}
}

#[cfg(not(test))]
/// Reset the VM via the keyboard controller
fn i8042_reset() -> ! {
    log!("i8042_reset...\n");
    loop {
        let mut good: u8 = 0x02;
        let mut i8042_command: Port<u8> = unsafe { Port::new(0x64) };
        while good & 0x02 > 0 {
            good = i8042_command.read();
        }
        i8042_command.write(0xFE);
    }
}

//#[cfg(not(test))]
/// Enable SSE2 for XMM registers (needed for EFI calling)
//fn enable_sse2() {
//    unsafe {
//        asm!("movq %cr0, %rax");
//        asm!("or $$0x2, %ax");
//        asm!("movq %rax, %cr0");
//        asm!("movq %cr4, %rax");
//        asm!("or $$0x600, %ax");
//        asm!("movq %rax, %cr4");
//    }
//}

#[cfg(not(test))]
#[no_mangle]
#[cfg_attr(target_os = "uefi", export_name = "efi_main")]
pub extern "win64" fn _start(hob: *const c_void) -> ! {

    log!("Starting UEFI hob - {:p}\n", hob);

    //enable_sse2();

    efi::enter_uefi(hob);

    //i8042_reset();
}
