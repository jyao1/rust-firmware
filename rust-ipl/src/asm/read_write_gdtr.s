# Copyright (c) 2021 Intel Corporation
# SPDX-License-Identifier: BSD-2-Clause-Patent

.section .text

# VOID
# EFIAPI
# sgdt_call (
#   OUT UINT64 addr
#   );
#
.global sgdt_call
sgdt_call:
    sgdt    (%rcx)
    ret

#  lgdt_call (
#        IN UINT64 addr
# )
.global lgdt_call
lgdt_call:
    lgdt    (%rcx)
    ret
