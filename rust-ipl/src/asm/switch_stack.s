# Copyright (c) 2020 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text

#  switch_stack_call(
#       entry_point: usize, // rcx
#       stack_top: usize,   // rdx
#       P1: usize,          // r8
#       P2: usize           // r9
#       );
.global switch_stack_call
switch_stack_call:

        subq $32, %rdx
        movq %rdx, %rsp
        movq %rcx, %rax
        movq %r8, %rcx
        movq %r9, %rdx
        call *%rax

        int $3
        jmp .
        ret
