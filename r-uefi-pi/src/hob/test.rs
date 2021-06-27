// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[test]
fn test_size() {
    assert_eq!(core::mem::size_of::<super::GenericHeader>(), 8);
    assert_eq!(core::mem::size_of::<super::HandoffInfoTable>(), 56);
    assert_eq!(core::mem::size_of::<super::MemoryAllocation>(), 48);
    assert_eq!(core::mem::size_of::<super::ResourceDescription>(), 48);
    assert_eq!(core::mem::size_of::<super::GuidExtension>(), 24);
    assert_eq!(core::mem::size_of::<super::FirmwareVolume>(), 24);
    assert_eq!(core::mem::size_of::<super::FirmwareVolume2>(), 56);
    assert_eq!(core::mem::size_of::<super::FirmwareVolume3>(), 64);
    assert_eq!(core::mem::size_of::<super::Cpu>(), 16);
    assert_eq!(core::mem::size_of::<super::MemoryPool>(), 8);
    assert_eq!(core::mem::size_of::<super::UefiCapsule>(), 24);
}