// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use core::fmt;
use r_uefi_pi::hob;
use scroll::Pread;
use scroll::Pwrite;

pub enum HobEnums {
    HandOff(hob::HandoffInfoTable),
    MemoryAllocation(hob::MemoryAllocation),
    ResourceDescription(hob::ResourceDescription),
    GuidExtension(hob::GuidExtension),
    FirmwareVolume(hob::FirmwareVolume),
    Cpu(hob::Cpu),
    MemoryPool(hob::MemoryPool),
    FirmwareVolume2(hob::FirmwareVolume2),
    UefiCapsule(hob::UefiCapsule),
    FirmwareVolume3(hob::FirmwareVolume3),
    Unused(hob::GenericHeader),
    EndOffHobList(hob::GenericHeader),
    Unknown(hob::GenericHeader),
}

pub struct HobList<'a> {
    data: &'a [u8],
    index: usize,
    offset: usize,
}

impl<'a> HobList<'a> {
    pub fn new(hob_list: &'a [u8]) -> Self {
        HobList {
            data: hob_list,
            index: 0,
            offset: 0,
        }
    }
    pub fn find_end_off_hob_offset(&self) -> Option<usize> {
        let handoff_hob = self.data.pread::<hob::HandoffInfoTable>(0).ok()?;

        let end_off_hob_offset = handoff_hob.efi_end_of_hob_list as usize;
        let result= self.data.pread::<hob::GenericHeader>(end_off_hob_offset).and_then(|header| {
            if hob::HobType::from(header.r#type) == hob::HobType::END_OF_HOB_LIST {
                Ok(end_off_hob_offset)
            } else {
                Err(scroll::Error::BadOffset(end_off_hob_offset))
            }
        }).map_err(|_e| {
            log::trace!("Got handoff_hob.efi_end_of_hob_list bad value. Fallback to search END_OF_LIST_HOB\n");
            let mut offset = 0usize;
            loop {
                let header = self.data.pread::<hob::GenericHeader>(offset).ok()?;

                if hob::HobType::from(header.r#type) == hob::HobType::END_OF_HOB_LIST {
                    return Some(offset);
                }
                offset += header.length as usize;
            }
        });
        match result {
            Ok(offset) => {Some(offset)}
            Err(search_offset) => {search_offset}
        }
    }
}

impl<'a> Iterator for HobList<'a> {
    type Item = HobEnums;
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        let header = self.data.pread::<hob::GenericHeader>(offset).ok()?;

        if hob::HobType::from(header.r#type) == hob::HobType::END_OF_HOB_LIST {
            return None;
        }

        let hob_enums = HobEnums::new(header, self.data, offset)?;
        self.offset += header.length as usize;
        self.index += 1;
        Some(hob_enums)
    }
}

impl HobEnums {
    pub fn new(header: hob::GenericHeader, hob: &[u8], offset: usize) -> Option<Self> {
        let res = match hob::HobType::from(header.r#type) {
            hob::HobType::HANDOFF => {
                let value = hob.pread::<hob::HandoffInfoTable>(offset).ok()?;
                HobEnums::HandOff(value)
            }
            hob::HobType::MEMORY_ALLOCATION => {
                let value = hob.pread::<hob::MemoryAllocation>(offset).ok()?;
                HobEnums::MemoryAllocation(value)
            }
            hob::HobType::RESOURCE_DESCRIPTOR => {
                let value = hob.pread::<hob::ResourceDescription>(offset).ok()?;
                HobEnums::ResourceDescription(value)
            }
            hob::HobType::GUID_EXTENSION => {
                let value = hob.pread::<hob::GuidExtension>(offset).ok()?;
                HobEnums::GuidExtension(value)
            }
            hob::HobType::FV => {
                let value = hob.pread::<hob::FirmwareVolume>(offset).ok()?;
                HobEnums::FirmwareVolume(value)
            }
            hob::HobType::CPU => {
                let value = hob.pread::<hob::Cpu>(offset).ok()?;
                HobEnums::Cpu(value)
            }
            hob::HobType::MEMORY_POOL => {
                let value = hob.pread::<hob::MemoryPool>(offset).ok()?;
                HobEnums::MemoryPool(value)
            }
            hob::HobType::FV2 => {
                let value = hob.pread::<hob::FirmwareVolume2>(offset).ok()?;
                HobEnums::FirmwareVolume2(value)
            }
            hob::HobType::LOAD_PEIM_UNUSED => HobEnums::Unknown(header),
            hob::HobType::UEFI_CAPSULE => {
                let value = hob.pread::<hob::UefiCapsule>(offset).ok()?;
                HobEnums::UefiCapsule(value)
            }
            hob::HobType::FV3 => {
                let value = hob.pread::<hob::FirmwareVolume3>(offset).ok()?;
                HobEnums::FirmwareVolume3(value)
            }
            hob::HobType::END_OF_HOB_LIST => HobEnums::EndOffHobList(header),
            hob::HobType::UNUSED => HobEnums::Unused(header),
            hob::HobType::Unknown(_) => HobEnums::Unknown(header),
        };
        Some(res)
    }
}

impl fmt::Debug for HobEnums {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HobEnums::HandOff(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!(
                    "{:?}\t\t\t{:010x}..{:010x}..{:010x}..{:010x}\n",
                    t,
                    value.efi_memory_bottom,
                    value.efi_free_memory_bottom,
                    value.efi_free_memory_top,
                    value.efi_memory_top
                ))
            }
            HobEnums::MemoryAllocation(value) => {
                let t = hob::HobType::from(value.header.r#type);
                let descriptor = value.alloc_descriptor;
                f.write_fmt(format_args!(
                    "{:?}\t{:010x}..{:010x} {:?} {:x}\n",
                    t,
                    descriptor.memory_base_address,
                    descriptor.memory_base_address + descriptor.memory_length,
                    descriptor.name,
                    descriptor.memory_type,
                ))
            }
            HobEnums::ResourceDescription(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!(
                    "{:?}\t{:010x}..{:010x} {:?}\t{:?} {:x}\n",
                    t,
                    value.physical_start,
                    value.physical_start + value.resource_length,
                    hob::ResourceType::from(value.resource_type),
                    value.owner,
                    value.resource_attribute
                ))
            }
            HobEnums::GuidExtension(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t{:?}\n", t, value.name))
            }
            HobEnums::FirmwareVolume(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::Cpu(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::MemoryPool(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t{:010x}\n", t, value.header.length))
            }
            HobEnums::FirmwareVolume2(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::UefiCapsule(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::FirmwareVolume3(value) => {
                let t = hob::HobType::from(value.header.r#type);
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::Unknown(value) => {
                let t = value.r#type;
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::Unused(value) => {
                let t = value.r#type;
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
            HobEnums::EndOffHobList(value) => {
                let t = value.r#type;
                f.write_fmt(format_args!("{:?}\t\t\n", t))
            }
        }
    }
}

pub struct HobListMut<'a> {
    data: &'a mut [u8],
}

impl<'a> HobListMut<'a> {
    pub fn new(hob_list: &'a mut [u8]) -> Self {
        HobListMut {
            data: hob_list,
        }
    }

    pub fn add(&mut self, hob_buffer: &[u8]) -> bool {
        let end_of_hob_offset = HobList::new(self.data).find_end_off_hob_offset();

        if end_of_hob_offset.is_none() {
            return false;
        }
        let end_of_hob_offset = end_of_hob_offset.unwrap();

        if end_of_hob_offset + hob_buffer.len() + core::mem::size_of::<hob::GenericHeader>() > self.data.len() {
            return false;
        }

        self.data[end_of_hob_offset..end_of_hob_offset+hob_buffer.len()].copy_from_slice(hob_buffer);

        let end_of_hob_offset = end_of_hob_offset+hob_buffer.len();

        // write END_OF_LIST hob
        let end_of_hob =  hob::GenericHeader::new(hob::HobType::END_OF_HOB_LIST, core::mem::size_of::<hob::GenericHeader>());
        let end_of_hob_size = self.data.pwrite::<hob::GenericHeader>(end_of_hob, end_of_hob_offset).unwrap();
        assert_eq!(end_of_hob_size, core::mem::size_of::<hob::GenericHeader>());

        // write handoftable
        let mut hand_of_hob = self.data.pread::<hob::HandoffInfoTable>(0).unwrap();
        hand_of_hob.efi_end_of_hob_list = (self.data as *const [u8] as *const u8 as usize + end_of_hob_offset) as u64;
        self.data.pwrite::<hob::HandoffInfoTable>(hand_of_hob, 0).unwrap();

        true
    }
}

pub fn dump_hob(hob_list: &[u8]) {
    for h in HobList::new(hob_list).filter(|h| -> bool {
        match h {
            HobEnums::MemoryPool(_) => {false}
            _ => true
        }
    }) {
        log::info!("{:?}", h);
    }
}

/// used for data storage (stack/heap/pagetable/eventlog/...)
pub fn get_system_memory_size_below_4gb(hob_list: &[u8]) -> u64 {
    let mut tolum = 0;
    for h in HobList::new(hob_list) {
        if let HobEnums::ResourceDescription(resource_hob) = h {
            if let hob::ResourceType::SYSTEM_MEMORY = hob::ResourceType::from(resource_hob.resource_type) {
                if resource_hob.resource_attribute.intersects(r_uefi_pi::hob::ResourceAttributeType::TESTED) {
                    let end = resource_hob.physical_start + resource_hob.resource_length;
                    if end > tolum {
                        tolum = end;
                    }
                }
            }
        }
    }
    tolum
}

/// used for page table setup
pub fn get_total_memory_top(hob_list: &[u8]) -> u64 {
    let mut value = 0;
    for h in HobList::new(hob_list) {
        if let HobEnums::ResourceDescription(resource_hob) = h {
            match hob::ResourceType::from(resource_hob.resource_type) {
                hob::ResourceType::SYSTEM_MEMORY | hob::ResourceType::MEMORY_MAPPED_IO => {
                    let end = resource_hob.physical_start + resource_hob.resource_length;
                    if end > value {
                        value = end;
                    }
                }
                _ => {}
            }
        }
    }
    value
}

pub fn get_hob_total_size(hob: &[u8]) -> Option<usize> {
    let hob_list = HobList::new(hob);
    let offset = hob_list.find_end_off_hob_offset()?;
    Some(offset + core::mem::size_of::<hob::GenericHeader>())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_hob_2g_example() {
        use std::io::Write;
        let _res = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format(|buf, record| write!(buf, "{}", record.args()))
            .try_init();

        #[path = "../../../test_data/fsp_hob_data.rs"]
        mod fsp_hob_data;
        let hob_list = &fsp_hob_data::FSP_HOB_2G_EXAMPLE[..];
        println!("fsp_hob: {}", hob_list.len());
        super::dump_hob(hob_list);
    }

    #[test]
    fn test_hob_8g_fsp_example() {
        use std::io::Write;
        let _res = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format(|buf, record| write!(buf, "{}", record.args()))
            .try_init();

        #[path = "../../../test_data/fsp_hob_data.rs"]
        mod fsp_hob_data;
        let hob_list = &fsp_hob_data::FSP_M_INIT_8G_HOB_EXAMPLE[..];
        test_hob_8g_fsp_m_example(hob_list);
        let hob_list = &fsp_hob_data::FSP_S_INIT_8G_HOB_EXAMPLE[..];
        test_hob_8g_fsp_s_example(hob_list);

        test_add(hob_list);
    }

    fn test_hob_8g_fsp_m_example(hob_list: &[u8]) {
        log::info!("fsp_hob: {}\n", hob_list.len());
        super::dump_hob(hob_list);
        log::info!("tolum: {:x}\n", super::get_system_memory_size_below_4gb(hob_list));
    }

    fn test_hob_8g_fsp_s_example(hob_list: &[u8]) {
        log::info!("fsp_hob: {}\n", hob_list.len());
        super::dump_hob(hob_list);
        log::info!("tolum: {:x}\n", super::get_system_memory_size_below_4gb(hob_list));
    }

    fn test_add(hob_list: &[u8]) {
        use r_uefi_pi::hob;
        use r_efi::efi;
        use scroll::Pwrite;
        let hob_list_size = super::get_hob_total_size(hob_list).unwrap();
        let hob_list = &hob_list[..hob_list_size];

        super::dump_hob(hob_list);

        let stack_hob = hob::MemoryAllocation {
            header: hob::GenericHeader::new(hob::HobType::MEMORY_ALLOCATION, core::mem::size_of::<hob::MemoryAllocation>()),
            alloc_descriptor: hob::MemoryAllocationHeader {
                name: hob::Guid::from_fields(1, 2, 3, 0, 0, &[0,0,0,0,0,0]),
                memory_base_address: 0x7df00000 as u64,
                memory_length: 0x800000 as u64,
                memory_type: efi::MemoryType::BootServicesData as u32,
                reserved: [0u8; 4],
            },
        };
        let stack_hob_buffer = &mut [0u8; core::mem::size_of::<hob::MemoryAllocation>()];
        let mut new_hob_list = Vec::<u8>::new();
        new_hob_list.extend_from_slice(hob_list);
        new_hob_list.extend_from_slice(stack_hob_buffer);

        let writen = stack_hob_buffer.pwrite::<hob::MemoryAllocation>(stack_hob, 0).unwrap();
        assert_eq!(writen, stack_hob_buffer.len());

        let hobs_list = super::HobList::new(new_hob_list.as_slice());
        let offset = hobs_list.find_end_off_hob_offset().unwrap();
        assert_eq!(offset+8, hob_list.len());

        let mut hobs_mut = super::HobListMut::new(new_hob_list.as_mut_slice());

        assert!(hobs_mut.add(stack_hob_buffer));

        log::info!("\nAfter add stack hob: \n");
        super::dump_hob(new_hob_list.as_slice());
    }

    #[test]
    fn test_hob_func() {
        use std::io::Write;
        let _res = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format(|buf, record| write!(buf, "{}", record.args()))
            .try_init();

        #[path = "../../../test_data/fsp_hob_data.rs"]
        mod fsp_hob_data;
        let hob_list = &fsp_hob_data::FSP_S_INIT_8G_HOB_EXAMPLE[..];
        test_filter(hob_list);
    }

    fn test_filter(hob_list: &[u8]) {
        let hob_list = super::HobList::new(hob_list);
        let hob_list = hob_list.into_iter().filter(|hob| -> bool {
            match hob {
                super::HobEnums::MemoryPool(_) => {false}
                super::HobEnums::Unknown(_) => {false}
                _=>true
            }
        });
        for h in hob_list {
            log::info!("{:?}", h);
        }
    }
}
