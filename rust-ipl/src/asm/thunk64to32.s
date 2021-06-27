#
# Copyright (c) 2021, Intel Corporation. All rights reserved.<BR>
# SPDX-License-Identifier: BSD-2-Clause-Patent
#
#
# Module Name:
#
#    thunk64to32.s
#
# Abstract:
#
#   This is the assembly code to transition from long mode to compatibility mode to execute 32-bit code and then
#   transit back to long mode.
#
# -------------------------------------------------------------------------------
    .section .text
# ----------------------------------------------------------------------------
# Procedure:    AsmExecute32BitCode
#
# Input:        None
#
# Output:       None
#
# Prototype:    UINT32
#               AsmExecute32BitCode (
#                 IN UINT64           Function,
#                 IN UINT64           Param1,
#                 IN UINT64           Param2,
#                 IN IA32_DESCRIPTOR  *InternalGdtr
#                 )#
#
#
# Description:  A thunk function to execute 32-bit code in long mode.
#
# ----------------------------------------------------------------------------

.set    LINEAR_CS64_SEL,      0x18
.set    LINEAR_CS32_SEL,      0x10
.set    LINEAR_DS_SEL,          0x8

.global  AsmExecute32BitCode
AsmExecute32BitCode:
AsmExecute32BitCodeStart:
    #
    # save IFLAG and disable it
    #
    pushfq
    cli

    #
    # save original GDTR and CS
    #
    movl    %ds, %eax
    pushq   %rax
    movl    %cs, %eax
    push    %rax
    subq    $0x10, %rsp
    sgdt    (%rsp)
    #
    # load internal GDT
    #
    lgdt    (%r9)
    #
    # Save general purpose register and rflag register
    #
    pushfq
    pushq    %rdi
    pushq    %rsi
    pushq    %rbp
    pushq    %rbx

    #
    # save CR3
    #
    movq     %cr3, %rax
    movq     %rax, %rbp

    #
    # Prepare the CS and return address for the transition from 32-bit to 64-bit mode
    #
    movq    $LINEAR_CS64_SEL, %rax
    shlq    $32, %rax
    lea     ReloadCS(%rip), %r9
    orq     %r9, %rax
    pushq   %rax
    #
    # Save parameters for 32-bit function call
    #
    movq     %r8, %rax
    shlq     $32, %rax
    orq      %rdx, %rax
    pushq    %rax
    #
    # save the 32-bit function entry and the return address into stack which will be
    # retrieve in compatibility mode.
    #
    lea     ReturnBack(%rip), %rax
    shlq     $32, %rax
    orq      %rcx, %rax
    pushq    %rax

    #
    # let rax save DS
    #
    movq     $LINEAR_DS_SEL, %rax

    #
    # Change to Compatible Segment
    #
    movq    $LINEAR_CS32_SEL, %rcx
    shlq    $32, %rcx
    lea     Compatible(%rip), %rdx
    or      %rdx, %rcx
    pushq   %rcx
    lret

Compatible:
    # reload DS/ES/SS to make sure they are correct referred to current GDT
    movw     %ax, %ds
    movw     %ax, %es
    movw     %ax, %ss

    #
    # Disable paging
    #
    movq    %cr0, %rcx
    btc     $31,  %ecx
    movq    %rcx, %cr0
    #
    # Clear EFER.LME
    #
    movl     $0xC0000080, %ecx
    rdmsr
    btc     $8, %eax
    wrmsr

    #
    # clear CR4 PAE
    #
    movq     %cr4, %rax
    btc      $5, %eax
    movq      %rax, %cr4

# Now we are in protected mode
    #
    # Call 32-bit function. Assume the function entry address and parameter value is less than 4G
    #
    popq    %rax                 # Here is the function entry
    #
    # Now the parameter is at the bottom of the stack,  then call in to IA32 function.
    #
    jmp   *%rax
ReturnBack:

    movl   %eax, %ebx
    popq   %rcx
    popq   %rcx

    #
    # restore CR4 PAE
    #
    movq     %cr4, %rax
    bts     $5, %eax
    movq     %rax, %cr4

    #
    # restore CR3
    #
    movl     %ebp, %eax
    movq     %rax, %cr3

    #
    # Set EFER.LME to re-enable ia32-e
    #
    movl     $0xC0000080, %ecx
    rdmsr
    bts     $8, %eax
    wrmsr
    #
    # Enable paging
    #
    movq     %cr0, %rax
    bts     $31, %eax
    movq     %rax, %cr0
# Now we are in compatible mode

    #
    # Reload cs register
    #
    lret
ReloadCS:
    #
    # Now we're in Long Mode
    #
    #
    # Restore C register and eax hold the return status from 32-bit function.
    # Note: Do not touch rax from now which hold the return value from IA32 function
    #
    movl     %ebx, %eax
    popq     %rbx
    popq     %rbp
    popq     %rsi
    popq     %rdi
    popfq
    #
    # Switch to original GDT and CS. here rsp is pointer to the original GDT descriptor.
    #
    lgdt    (%rsp)
    #
    # drop GDT descriptor in stack
    #
    add     $0x10, %rsp
    #
    # switch to original CS and GDTR
    #
    popq     %r9
    shlq     $32, %r9
    lea      END_0(%rip), %rcx
    orq      %r9, %rcx
    pushq    %rcx
    lret
END_0:
    #
    # Reload original DS/ES/SS
    #
    popq     %rcx
    movw     %cx, %ds
    movw     %cx, %es
    movw     %cx, %ss

    #
    # Restore IFLAG
    #
    popfq

    ret
AsmExecute32BitCodeEnd:
