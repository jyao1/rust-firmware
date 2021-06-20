;------------------------------------------------------------------------------
; @file
; Fsp-T  TempRamInit Wrapper
;
; Copyright (c) 2008 - 2009, Intel Corporation. All rights reserved.<BR>
; SPDX-License-Identifier: BSD-2-Clause-Patent
;
;------------------------------------------------------------------------------

%define FSP_HEADER_TEMPRAMINIT_OFFSET 0x30

; ResetVectorParams offset 0x4 contains FSP-T TempRamInit API Params
%define FSP_T_TEMP_RAM_INIT_PARAM_OFFSET  LOADED_RESET_VECTOR_BASE + 0x4

BITS    32

TempRamInitStack:
    DD      ADDR_OF(TempRamInitDone)
    DD      FSP_T_TEMP_RAM_INIT_PARAM_OFFSET

FspWrapperTempRamInit:
    ; Modify:
    ;   EBX, EAX, ECX, EDX, ESI

    ; Output:
    ;
    ; FSP-T NEM returned range
    ;   ECX: NEM stack base
    ;   EDX: NEM stack top
    ;

    ;
    ; Get FSP-T base in EAX
    ;
    mov       eax, LOADED_FSP_T_BASE

    ;
    ; Find the fsp info header
    ; Jump to TempRamInit API
    ;
    add     eax, dword [eax + 094h + FSP_HEADER_TEMPRAMINIT_OFFSET]
    mov     esp, ADDR_OF(TempRamInitStack)
    jmp     eax

FspApiSuccess:
    OneTimeCallRet  FspWrapperTempRamInit

TempRamInitDone:
    cmp     eax, 8000000Eh      ;Check if EFI_NOT_FOUND returned. Error code for Microcode Update not found.
    je      FspApiSuccess       ;If microcode not found, don't hang, but continue.

    cmp     eax, 0              ;Check if EFI_SUCCESS returned.
    jz      FspApiSuccess

    ; FSP API failed:
    jmp     $
