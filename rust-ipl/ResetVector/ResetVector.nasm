;------------------------------------------------------------------------------
; @file
; This file includes all other code files to assemble the reset vector code
;
; Copyright (c) 2008 - 2013, Intel Corporation. All rights reserved.<BR>
; SPDX-License-Identifier: BSD-2-Clause-Patent
;
;------------------------------------------------------------------------------

;
; If neither ARCH_IA32 nor ARCH_X64 are defined, then try to include
; Base.h to use the C pre-processor to determine the architecture.
;

%ifdef ARCH_IA32
  %ifdef ARCH_X64
    %fatal "Only one of ARCH_IA32 or ARCH_X64 can be defined."
  %endif
%elifdef ARCH_X64
%else
  %fatal "Either ARCH_IA32 or ARCH_X64 must be defined."
%endif

%include "CommonMacros.inc"

%include "PostCodes.inc"

%ifdef DEBUG_PORT80
  %include "Port80Debug.asm"
%elifdef DEBUG_SERIAL
  %include "SerialDebug.asm"
%else
  %include "DebugDisabled.asm"
%endif

%include "FspWrapper.asm"

%ifdef ARCH_X64
%include "PageTables.asm"
%include "Ia32/Flat32ToFlat64.asm"
%endif

%include "Ia16/Real16ToFlat32.asm"
%include "Ia16/Init16.asm"

%include "Main.asm"

%include "Ia16/ResetVectorVtf0.asm"

