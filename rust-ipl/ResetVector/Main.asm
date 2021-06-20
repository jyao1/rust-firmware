;------------------------------------------------------------------------------
; @file
; Main routine of the pre-SEC code up through the jump into SEC
;
; Copyright (c) 2008 - 2009, Intel Corporation. All rights reserved.<BR>
; SPDX-License-Identifier: BSD-2-Clause-Patent
;
;------------------------------------------------------------------------------

;
; Minimal stack size requried by sec
;
%define SEC_MINIMAL_STACK_SIZE 0x1000

BITS    16

;
; Modified:  EBX, ECX, EDX, EBP
;
; @param[in,out]  RAX/EAX  Initial value of the EAX register
;                          (BIST: Built-in Self Test)
; @param[in,out]  DI       'BP': boot-strap processor, or
;                          'AP': application processor
; @param[out]     RBP/EBP  Address of Boot Firmware Volume (BFV)
; @param[out]     DS       Selector allowing flat access to all addresses
; @param[out]     ES       Selector allowing flat access to all addresses
; @param[out]     FS       Selector allowing flat access to all addresses
; @param[out]     GS       Selector allowing flat access to all addresses
; @param[out]     SS       Selector allowing flat access to all addresses
;
; @return         None  This routine jumps to SEC and does not return
;
Main16:
    OneTimeCall EarlyInit16

    ;
    ; Transition the processor from 16-bit real mode to 32-bit flat mode
    ;
    OneTimeCall TransitionFromReal16To32BitFlat

BITS    32

    ; TBD: call FSP-T to initialize Temp memory
    ; return ecx, edx
    ;
    OneTimeCall FspWrapperTempRamInit

    ;
    ; Initialize Temp Stack
    ;
    mov     ebx, esp
    mov     eax, edx
    and     eax, 0xfffff000
    ; reserved for paging and sec
    ; at least 0x1000 for sec stack
    sub     eax, PAGE_REGION_SIZE  + SEC_MINIMAL_STACK_SIZE
    ;
    ; Make Sure enough temp ram
    ;
    cmp     eax, ecx
    jg      GetEnoughTempRamOk
    debugShowPostCode POSTCODE_SEC_TMP_RAM_TOO_SMALL
    jmp     $
GetEnoughTempRamOk:

    add     eax, SEC_MINIMAL_STACK_SIZE
    mov     esp, eax
    nop
    push    ebx         ; save Initial value of the EAX register
    push    eax         ; save TempPageTableBase/TempStackBase to stack
    push    ecx         ; save Temp memory start
    push    edx         ; save Temp memory end

    ;
    ; LOADED_RESET_VECTOR_BASE store params requried by reset vector
    ; See: rust-firmware-tool/src/main.rs: ResetVectorParams
    ;
    mov     eax, LOADED_RESET_VECTOR_BASE
    mov     esi, dword [eax]
    ;
    ; ESI - SEC Core entry point
    ;

%ifdef ARCH_IA32
    ; TBD: Impl IA32 arch
    debugShowPostCode POSTCODE_SEC_NOT_FOUND
    jmp $
%else
    ; Create page tables
    ;   ECX: Page base
    mov     eax, edx
    and     eax, 0xfffff000
    sub     eax, PAGE_REGION_SIZE
    mov     ecx, eax

    push    esi
    push    ebp
    push    eax
    OneTimeCall  PreparePagingTable
    pop     eax
    pop     ebp
    pop     esi
    mov     cr3, eax

    ;
    ; Transition the processor from 32-bit flat mode to 64-bit flat mode
    ;
    OneTimeCall Transition32FlatTo64Flat

BITS    64

    ;
    ; Some values were calculated in 32-bit mode.  Make sure the upper
    ; 32-bits of 64-bit registers are zero for these values.
    ;
    mov     rax, 0x00000000ffffffff
    and     rdx, rax
    and     rsi, rax
    and     rbp, rax
    and     rsp, rax
    and     r8,  rax
    and     r9,  rax

    ;
    ; RSI - SEC Core entry point
    ;

    ;
    ; Setup parameters and call SecCoreStartupWithStack
    ;   rcx: TempRamBase
    ;   rdx: TempRamTop
    ;   r8:  TempPageTableBase or stack base
    ;   r9:  initial EAX value into the EAX register
    ; Restore initial EAX value into the EAX register
    ;

    mov     ecx, dword [rsp + 4]
    mov     edx, dword [rsp]
    mov     r8d, dword [rsp + 8]
    mov     r9d, dword [rsp + 12]
    sub     rsp, 0x20
    call    rsi

%endif
