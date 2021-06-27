// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use x86::dtables;

global_asm!(include_str!("switch_stack.s"));
global_asm!(include_str!("thunk64to32.s"));
global_asm!(include_str!("read_write_gdtr.s"));
global_asm!(include_str!("read_write_idtr.s"));

extern "win64" {
    fn switch_stack_call(entry_point: usize, stack_top: usize, p1: usize, p2: usize);
    fn AsmExecute32BitCode(function_entry: usize, param1: usize, param2: usize, gdtr: usize) -> usize;
    fn lidt_call(idtr: usize);
    fn sgdt_call(gdtr: usize);
}

pub fn switch_stack(entry_point: usize, stack_top: usize, p1: usize, p2: usize) {
    unsafe {
        switch_stack_call(entry_point, stack_top, p1, p2)
    }
    panic!("not possible");
}

pub fn execute_32bit_code(entry_point: usize, param1: usize, param2: usize) -> usize {
    log::trace!("execute 32bit code - entry: 0x{:x}, param1: 0x{:x}, param2: 0x{:x}\n", entry_point, param1, param2);
    unsafe {
        // Let FSP to setup the IDT
        let idtr_null = x86::dtables::DescriptorTablePointer {
            base: core::ptr::null::<usize>(),
            limit: 0xffff,
        };
        lidt_call(&idtr_null as *const dtables::DescriptorTablePointer::<usize> as usize);

        // Use current GDT
        let gdtr = dtables::DescriptorTablePointer::<usize>::default();
        sgdt_call(&gdtr as *const dtables::DescriptorTablePointer::<usize> as usize);

        AsmExecute32BitCode(entry_point, param1, param2, &gdtr as *const dtables::DescriptorTablePointer::<usize> as usize)
    }
}
