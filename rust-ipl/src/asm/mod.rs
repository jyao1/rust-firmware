// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

global_asm!(include_str!("switch_stack.s"));

extern "win64" {
    fn switch_stack_call(entry_point: usize, stack_top: usize, p1: usize, p2: usize);
}

pub fn switch_stack(entry_point: usize, stack_top: usize, p1: usize, p2: usize) {
    unsafe {
        switch_stack_call(entry_point, stack_top, p1, p2)
    }
    panic!("not possible");
    loop {}
}