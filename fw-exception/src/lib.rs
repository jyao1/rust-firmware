// Copyright (c) 2020-2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#![no_std]
#![feature(llvm_asm)]
#![feature(naked_functions)]
#![feature(global_asm)]

mod asm;
mod idt;
mod interrupt;

pub fn setup_exception_handlers() {
    unsafe { idt::init() };
}
