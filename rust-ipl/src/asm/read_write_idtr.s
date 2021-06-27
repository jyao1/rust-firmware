# Copyright (c) 2021 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text

#  sidt_call (
#        OUT UINT64 addr
# )
.global sidt_call
sidt_call:
    sidt    (%rcx)
    ret


#  lidt_call (
#        IN UINT64 addr
# )
.global lidt_call
lidt_call:
    lidt    (%rcx)
    ret

