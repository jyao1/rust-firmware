// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/*
#![feature(naked_functions)]
#![feature(asm_sym, asm_const)]
#[macro_use]
use core::arch::asm;
use core::arch::global_asm;
*/
// global_asm!(include_str!("idt.s", inout("rcx") rcx));

/*
#[naked]
#[export_name = "sidt_call"]
#[link_section = ".text"]
unsafe extern "C" fn sidt_call() -> ! {
    let mut rcx: u32;
    asm!(
        ".global sidt_call",
        "sidt_call:",
        "sidt    ({rcx})",
        "ret",
        rcx = sym rcx,
        options(noreturn)
    );
}
*/
