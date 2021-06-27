// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub use crate::r_efi_wrapper::Guid;
use r_efi::efi::PhysicalAddress;
use scroll::{Pread, Pwrite};

mod hob_type;
mod resource_attribute;
mod resource_type;

pub use hob_type::HobType;
pub use resource_attribute::ResourceAttributeType;
pub use resource_type::ResourceType;

///
/// Describes the format and size of the data inside the HOB.
/// All HOBs must contain this generic HOB header.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct GenericHeader {
    ///
    /// Identifies the HOB data structure type.
    ///
    pub r#type: u16,
    ///
    /// The length in bytes of the HOB.
    ///
    pub length: u16,
    ///
    /// This field must always be set to zero.
    ///
    pub reserved: u32,
}

impl GenericHeader {
    pub fn new(r#type: HobType, length: usize) -> Self {
        GenericHeader {
            r#type: r#type.get_u16(),
            length: length as u16,
            reserved: 0,
        }
    }
}

///
/// Value of version  in EFI_HOB_HANDOFF_INFO_TABLE.
///
pub const EFI_HOB_HANDOFF_TABLE_VERSION: u32 = 0x0009;

///
/// Contains general state information used by the HOB producer phase.
/// This HOB must be the first one in the HOB list.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct HandoffInfoTable {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_HANDOFF.
    ///
    pub header: GenericHeader,
    ///
    /// The version number pertaining to the PHIT HOB definition.
    /// This value is four bytes in length to provide an 8-byte aligned entry
    /// when it is combined with the 4-byte BootMode.
    ///
    pub version: u32,
    ///
    /// The system boot mode as determined during the HOB producer phase.
    ///
    pub boot_mode: u32,
    ///
    /// The highest address location of memory that is allocated for use by the HOB producer
    /// phase. This address must be 4-KB aligned to meet page restrictions of UEFI.
    ///
    pub efi_memory_top: PhysicalAddress,
    ///
    /// The lowest address location of memory that is allocated for use by the HOB producer phase.
    ///
    pub efi_memory_bottom: PhysicalAddress,
    ///
    /// The highest address location of free memory that is currently available
    /// for use by the HOB producer phase.
    ///
    pub efi_free_memory_top: PhysicalAddress,
    ///
    /// The lowest address location of free memory that is available for use by the HOB producer phase.
    ///
    pub efi_free_memory_bottom: PhysicalAddress,
    ///
    /// The end of the HOB list.
    ///
    pub efi_end_of_hob_list: PhysicalAddress,
}

///
/// EFI_HOB_MEMORY_ALLOCATION_HEADER describes the
/// various attributes of the logical memory allocation. The type field will be used for
/// subsequent inclusion in the UEFI memory map.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct MemoryAllocationHeader {
    ///
    /// A GUID that defines the memory allocation region's type and purpose, as well as
    /// other fields within the memory allocation HOB. This GUID is used to define the
    /// additional data within the HOB that may be present for the memory allocation HOB.
    /// Type EFI_GUID is defined in InstallProtocolInterface() in the UEFI 2.0
    /// specification.
    ///
    pub name: Guid,
    ///
    /// The base address of memory allocated by this HOB. Type
    /// EFI_PHYSICAL_ADDRESS is defined in AllocatePages() in the UEFI 2.0
    /// specification.
    ///
    pub memory_base_address: PhysicalAddress,
    ///
    /// The length in bytes of memory allocated by this HOB.
    ///
    pub memory_length: u64,
    ///
    /// Defines the type of memory allocated by this HOB. The memory type definition
    /// follows the EFI_MEMORY_TYPE definition. Type EFI_MEMORY_TYPE is defined
    /// in AllocatePages() in the UEFI 2.0 specification.
    ///
    pub memory_type: u32,

    ///
    /// Padding for Itanium processor family
    ///
    pub reserved: [u8; 4],
}

///
/// Describes all memory ranges used during the HOB producer
/// phase that exist outside the HOB list. This HOB type
/// describes how memory is used, not the physical attributes of memory.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct MemoryAllocation {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_MEMORY_ALLOCATION.
    ///
    pub header: GenericHeader,
    ///
    /// An instance of the EFI_HOB_MEMORY_ALLOCATION_HEADER that describes the
    /// various attributes of the logical memory allocation.
    ///
    pub alloc_descriptor: MemoryAllocationHeader,
    //
    // Additional data pertaining to the "Name" Guid memory
    // may go here.
    //
}

///
/// Describes the memory stack that is produced by the HOB producer
/// phase and upon which all post-memory-installed executable
/// content in the HOB producer phase is executing.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct MemoryAllocationStack {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_MEMORY_ALLOCATION.
    ///
    pub header: GenericHeader,
    ///
    /// An instance of the EFI_HOB_MEMORY_ALLOCATION_HEADER that describes the
    /// various attributes of the logical memory allocation.
    ///
    pub alloc_descriptor: MemoryAllocationHeader,
}

///
/// Describes the resource properties of all fixed,
/// nonrelocatable resource ranges found on the processor
/// host bus during the HOB producer phase.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct ResourceDescription {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_RESOURCE_DESCRIPTOR.
    ///
    pub header: GenericHeader,
    ///
    /// A GUID representing the owner of the resource. This GUID is used by HOB
    /// consumer phase components to correlate device ownership of a resource.
    ///
    pub owner: Guid,
    ///
    /// The resource type enumeration as defined by EFI_RESOURCE_TYPE.
    ///
    pub resource_type: u32,
    ///
    /// Resource attributes as defined by EFI_RESOURCE_ATTRIBUTE_TYPE.
    ///
    pub resource_attribute: ResourceAttributeType,
    ///
    /// The physical start address of the resource region.
    ///
    pub physical_start: PhysicalAddress,
    ///
    /// The number of bytes of the resource region.
    ///
    pub resource_length: u64,
}

///
/// Allows writers of executable content in the HOB producer phase to
/// maintain and manage HOBs with specific GUID.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct GuidExtension {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_GUID_EXTENSION.
    ///
    pub header: GenericHeader,
    ///
    /// A GUID that defines the contents of this HOB.
    ///
    pub name: Guid,
    //
    // Guid specific data goes here
    //
}

///
/// Details the location of firmware volumes that contain firmware files.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct FirmwareVolume {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_FV.
    ///
    pub header: GenericHeader,
    ///
    /// The physical memory-mapped base address of the firmware volume.
    ///
    pub base_address: PhysicalAddress,
    ///
    /// The length in bytes of the firmware volume.
    ///
    pub length: u64,
}

///
/// Details the location of a firmware volume that was extracted
/// from a file within another firmware volume.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct FirmwareVolume2 {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_FV2.
    ///
    pub header: GenericHeader,
    //
    /// The physical memory-mapped base address of the firmware volume.
    ///
    pub base_address: PhysicalAddress,
    ///
    /// The length in bytes of the firmware volume.
    ///
    pub length: u64,
    ///
    /// The name of the firmware volume.
    ///
    pub fv_name: Guid,
    ///
    /// The name of the firmware file that contained this firmware volume.
    ///
    pub file_name: Guid,
}

///
/// Details the location of a firmware volume that was extracted
/// from a file within another firmware volume.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct FirmwareVolume3 {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_FV3.
    ///
    pub header: GenericHeader,
    ///
    /// The physical memory-mapped base address of the firmware volume.
    ///
    pub base_address: PhysicalAddress,
    ///
    /// The length in bytes of the firmware volume.
    ///
    pub length: u64,
    ///
    /// The authentication status.
    ///
    pub authentication_status: u32,
    ///
    /// TRUE if the FV was extracted as a file within another firmware volume.
    /// FALSE otherwise.
    ///
    pub extracted_fv: u8, // Boolean
    ///
    /// The name of the firmware volume.
    /// Valid only if IsExtractedFv is TRUE.
    ///
    pub fv_name: Guid,
    ///
    /// The name of the firmware file that contained this firmware volume.
    /// Valid only if IsExtractedFv is TRUE.
    ///
    pub file_name: Guid,
}

///
/// Describes processor information, such as address space and I/O space capabilities.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct Cpu {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_CPU.
    ///
    pub header: GenericHeader,
    ///
    /// Identifies the maximum physical memory addressability of the processor.
    ///
    pub size_of_memory_space: u8,
    ///
    /// Identifies the maximum physical I/O addressability of the processor.
    ///
    pub size_of_io_space: u8,
    ///
    /// This field will always be set to zero.
    ///
    pub reserved: [u8; 6],
}

///
/// Each UEFI capsule HOB details the location of a UEFI capsule. It includes a base address and length
/// which is based upon memory blocks with a EFI_CAPSULE_HEADER and the associated
/// CapsuleImageSize-based payloads. These HOB's shall be created by the PEI PI firmware
/// sometime after the UEFI UpdateCapsule service invocation with the
/// CAPSULE_FLAGS_POPULATE_SYSTEM_TABLE flag set in the EFI_CAPSULE_HEADER.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct UefiCapsule {
    ///
    /// The HOB generic header where Header.HobType = EFI_HOB_TYPE_UEFI_CAPSULE.
    ///
    pub header: GenericHeader,

    ///
    /// The physical memory-mapped base address of an UEFI capsule. This value is set to
    /// point to the base of the contiguous memory of the UEFI capsule.
    /// The length of the contiguous memory in bytes.
    ///
    pub base_address: PhysicalAddress,
    pub length: u64,
}

///
/// Describes pool memory allocations.
///
#[repr(C)]
#[derive(Copy, Clone, Debug, Pread, Pwrite)]
pub struct MemoryPool {
    ///
    /// The HOB generic header. Header.HobType = EFI_HOB_TYPE_MEMORY_POOL.
    ///
    pub header: GenericHeader,
}

#[cfg(test)]
mod test;