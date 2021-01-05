// Copyright © 2019 Intel Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::ffi::c_void;

use r_efi::efi::{AllocateType, Char16, Guid, MemoryType, Status};
use r_efi::protocols::device_path::Protocol as DevicePathProtocol;
use r_efi::protocols::file::Protocol as FileProtocol;
use r_efi::protocols::simple_file_system::Protocol as SimpleFileSystemProtocol;
use crate::efi::fat::Error as FatError;
use crate::efi::fat::DirectoryEntry;
use crate::efi::fat::FileType;

#[cfg(not(test))]
#[repr(C)]
pub struct FileDevicePathProtocol {
    pub device_path: DevicePathProtocol,
    pub filename: [u16; 64],
}

#[cfg(not(test))]
pub extern "win64" fn filesystem_open_volume(
    fs_proto: *mut SimpleFileSystemProtocol,
    file: *mut *mut FileProtocol,
) -> Status {
    // log!("EFI-STUB: open_volume start\n");
    let wrapper = container_of!(fs_proto, FileSystemWrapper, proto);
    let wrapper = unsafe { &*wrapper };

    if let Some(fw) = wrapper.create_root_file() {
        unsafe {
            *file = &mut (*fw).proto;
        }
        // log!("EFI-STUB: open_volume\n");
        Status::SUCCESS
    } else {
        log!("EFI-STUB: open_volume failed\n");
        Status::DEVICE_ERROR
    }
}

#[cfg(not(test))]
pub extern "win64" fn open(
    file_in: *mut FileProtocol,
    file_out: *mut *mut FileProtocol,
    path_in: *mut Char16,
    _: u64,
    _: u64,
) -> Status {
    let wrapper = container_of!(file_in, FileWrapper, proto);
    let wrapper = unsafe { &*wrapper };
    //if !wrapper.root {
    //    log!("Attempt to open file from non-root file is unsupported\n");
    //    return Status::UNSUPPORTED;
    //}

    let mut path = [0; 256];
    let length = crate::common::ucs2_as_ascii_length(path_in);
    crate::common::ucs2_to_ascii(path_in, &mut path);
    let path = unsafe { core::str::from_utf8_unchecked(&path) };
    let path = &path[0..length] as &str;
    log!("EFI_STUB - enter open - file_in address: {:x} - path: {}\n", file_in as *mut FileProtocol as u64, path);

    if path == "\\" {
        log!("EFI-STUB: path = \\\n");
        let fs_wrapper = unsafe { &(*wrapper.fs_wrapper) };
        let file_out_wrapper = fs_wrapper.create_root_file().unwrap();
        unsafe {
            *file_out = &mut (*file_out_wrapper).proto;
            return Status::SUCCESS;
        }
    }
    if path == "." {
        log!("EFI-STUB: path = .\n");
        unsafe{*file_out = file_in;}
        return Status::SUCCESS;
    }
    if path == ".." {
        log!("EFI-STUB: path = ..\n");
        if wrapper.root {
            return Status::NOT_FOUND;
        }

        let protocol_address = unsafe{(*wrapper).parent};
        unsafe{
            *file_out = protocol_address as *mut c_void as *mut FileProtocol;
        }
        return Status::SUCCESS;
    }

    match wrapper.fs.open(path) {
        Ok(f) => {
            log!("EFI-STUB: file protocol open function ok\n");
            let fs_wrapper = unsafe { &(*wrapper.fs_wrapper) };
            if let Some(file_out_wrapper) = fs_wrapper.create_file(false) {
                let filename = unsafe{core::str::from_utf8_unchecked(&f.name)};
                log!("EFI-STUB: file protocol open function filename: {:?}, filesize: {:?}\n", filename, f.size);
                match f.file_type {
                    FileType::Directory => {
                        unsafe{
                            (*file_out_wrapper).directory = wrapper.fs.get_directory(f.cluster).unwrap();
                            (*file_out_wrapper).dir_entry = DirectoryEntry {
                                name: f.name,
                                cluster: f.cluster,
                                file_type:  FileType::Directory,
                                size: f.size,
                                long_name: f.long_name,
                            };
                            (*file_out_wrapper).parent = unsafe{&(*wrapper).proto as *const FileProtocol as u64};
                        }
                    }
                    FileType::File => {
                        unsafe {
                            (*file_out_wrapper).file = wrapper.fs.get_file(f.cluster, f.size).unwrap();
                            (*file_out_wrapper).dir_entry = DirectoryEntry {
                                name: f.name,
                                cluster: f.cluster,
                                file_type:  FileType::File,
                                size: f.size,
                                long_name: f.long_name,
                            };
                        }
                    }
                }
                unsafe {
                    *file_out = &mut (*file_out_wrapper).proto;
                }
                log!("EFI-STUB: file.rs open successful\n");
                Status::SUCCESS
            } else {
                log!("EFI-STUB: file.rs open failed device_error\n");
                Status::DEVICE_ERROR
            }
        },
        Err(FatError::NotFound) => {
            log!("EFI-STUB: open failed not found {:?}\n", path);
            Status::NOT_FOUND
        },
        Err(_) => {
            log!("EFI-STUB: file.rs open failed device_error\n");
            Status::DEVICE_ERROR
        },
    }
}

#[cfg(not(test))]
pub extern "win64" fn close(proto: *mut FileProtocol) -> Status {
    let wrapper = container_of!(proto, FileWrapper, proto);
    super::ALLOCATOR
        .lock()
        .free_pages(&wrapper as *const _ as u64)
}

#[cfg(not(test))]
pub extern "win64" fn delete(_: *mut FileProtocol) -> Status {
    crate::log!("delete unsupported");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn read(file: *mut FileProtocol, size: *mut usize, buf: *mut c_void) -> Status {
    let wrapper = container_of_mut!(file, FileWrapper, proto);
    let wrapper_value = unsafe{&(*wrapper)};
    match wrapper_value.dir_entry.file_type {
        crate::fat::FileType::File => {
            let mut current_offset = 0;
            let mut bytes_remaining = unsafe { *size };

            loop {
                use crate::fat::Read;
                let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, *size) };

                let mut data: [u8; 512] = [0; 512];
                unsafe {
                    match (*wrapper).file.read(&mut data) {
                        Ok(bytes_read) => {
                            buf[current_offset..current_offset + bytes_read as usize]
                                .copy_from_slice(&data[0..bytes_read as usize]);
                            current_offset += bytes_read as usize;

                            if bytes_remaining <= bytes_read as usize {
                                *size = current_offset;
                                return Status::SUCCESS;
                            }
                            bytes_remaining -= bytes_read as usize;
                        }
                        Err(_) => {
                            return Status::DEVICE_ERROR;
                        }
                    }
                }
            }
        }

        crate::fat::FileType::Directory => {
            let info = buf as *mut FileInfo;
            unsafe {
                let fs = (*wrapper).fs;
                let cluster = (*wrapper).dir_entry.cluster;
                let directory = &mut (*wrapper).directory;
                match directory.next_entry() {
                    Err(crate::fat::Error::EndOfFile) => {
                        (*info).size = 0;
                        (*info).attribute = 0;
                        (*info).file_name[0] = 0;
                        (*size) = 0;
                        return Status::SUCCESS;
                    }
                    Err(e) => {
                        log!("EFI-STUB: next_entry device error\n");
                        return Status::DEVICE_ERROR;
                    }
                    Ok(de) => {
                        let mut long_name = de.long_name;
                        if crate::common::ascii_length(&long_name as *const u8, 255) == 0 {
                            for i in 0..11 {
                                long_name[i] = de.name[i];
                            }
                        }
                        let mut fname = unsafe{core::str::from_utf8_unchecked(&long_name)};
                        let filename = &fname as &str;
                        crate::common::ascii_to_ucs2(filename, &mut (*info).file_name);
                        match de.file_type {
                            crate::fat::FileType::File => {
                                (*info).size = core::mem::size_of::<FileInfo>() as u64;
                                (*info).file_size = de.size.into();
                                (*info).physical_size = de.size.into();
                                (*info).attribute = 0x20;
                            }
                            crate::fat::FileType::Directory => {
                                // TODO: calculate the size of directory.
                                (*info).size = core::mem::size_of::<FileInfo>() as u64;
                                (*info).file_size = 4096;
                                (*info).physical_size = 4096;
                                (*info).attribute = 0x10;
                            }
                        }
                        return Status::SUCCESS;
                    }
                }
            }
            return Status::SUCCESS;
        }
    }


}

#[cfg(not(test))]
pub extern "win64" fn write(_: *mut FileProtocol, _: *mut usize, _: *mut c_void) -> Status {
    crate::log!("write unsupported");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn get_position(_: *mut FileProtocol, _: *mut u64) -> Status {
    crate::log!("get_position unsupported");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn set_position(_: *mut FileProtocol, _: u64) -> Status {
    // TODO: set position for opened file and opend directory.
    // crate::log!("set_position todo\n");
    Status::SUCCESS
}

#[cfg(not(test))]
#[repr(packed)]
struct FileInfo {
    size: u64,
    file_size: u64,
    physical_size: u64,
    _create_time: r_efi::system::Time,
    _last_access_time: r_efi::system::Time,
    _modification_time: r_efi::system::Time,
    attribute: u64,
    file_name: [Char16; 256],
}

#[cfg(not(test))]
pub extern "win64" fn get_info(
    file: *mut FileProtocol,
    guid: *mut Guid,
    info_size: *mut usize,
    info: *mut c_void,
) -> Status {
    let wrapper = container_of!(file, FileWrapper, proto);
    if unsafe { *guid } == r_efi::protocols::file::INFO_ID {
        if unsafe { *info_size } < core::mem::size_of::<FileInfo>() {
            unsafe { *info_size = core::mem::size_of::<FileInfo>() };
            Status::BUFFER_TOO_SMALL
        } else {
            let info = info as *mut FileInfo;
            use crate::fat::Read;
            unsafe {
                let mut long_name = (*wrapper).dir_entry.long_name;
                if crate::common::ascii_length(&long_name as *const u8, 255) == 0 {
                    for i in 0..11 {
                        long_name[i] = (*wrapper).dir_entry.name[i];
                    }
                }
                let filename = unsafe{core::str::from_utf8_unchecked(&long_name)};
                //log!("EFI-STUB: get_info: dir_entry.name: {:?}, dir_entry.long_name: {:?}\n", (*wrapper).dir_entry.name, filename);
                let filename = &filename[0..255] as &str;
                crate::common::ascii_to_ucs2(filename, &mut (*info).file_name);
                match (*wrapper).dir_entry.file_type {
                    crate::fat::FileType::File => {
                        (*info).size = core::mem::size_of::<FileInfo>() as u64;
                        (*info).file_size = (*wrapper).file.get_size().into();
                        (*info).physical_size = (*wrapper).file.get_size().into();
                        (*info).attribute = 0x20;
                    }
                    crate::fat::FileType::Directory => {
                        (*info).size = core::mem::size_of::<FileInfo>() as u64;
                        (*info).file_size = 4096;
                        (*info).physical_size = 4096;
                        (*info).attribute = 0x10;
                    }
                }
            }

            Status::SUCCESS
        }
    } else {
        crate::log!("get_info unsupported");
        Status::UNSUPPORTED
    }
}

#[cfg(not(test))]
pub extern "win64" fn set_info(
    _: *mut FileProtocol,
    _: *mut Guid,
    _: usize,
    _: *mut c_void,
) -> Status {
    crate::log!("set_info unsupported");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
pub extern "win64" fn flush(_: *mut FileProtocol) -> Status {
    crate::log!("flush unsupported");
    Status::UNSUPPORTED
}

#[cfg(not(test))]
struct FileWrapper<'a> {
    fs: &'a crate::fat::Filesystem<'a>,
    proto: FileProtocol,
    file: crate::fat::File<'a>,
    fs_wrapper: *const FileSystemWrapper<'a>,
    root: bool,
    dir_entry: DirectoryEntry,
    directory: crate::fat::Directory<'a>,
    parent: u64,
}

#[cfg(not(test))]
#[repr(C)]
pub struct FileSystemWrapper<'a> {
    hw: super::HandleWrapper,
    fs: &'a crate::fat::Filesystem<'a>,
    pub proto: SimpleFileSystemProtocol,
    pub block_part_id: Option<u32>,
}

#[cfg(not(test))]
impl<'a> FileSystemWrapper<'a> {

    // alloc a new FileWrapper
    fn create_file(&self, root: bool) -> Option<*mut FileWrapper> {
        let size = core::mem::size_of::<FileWrapper>();
        let (status, new_address) = super::ALLOCATOR.lock().allocate_pages(
            AllocateType::AllocateAnyPages,
            MemoryType::LoaderData,
            ((size + super::PAGE_SIZE as usize - 1) / super::PAGE_SIZE as usize) as u64,
            0 as u64,
        );

        if status == Status::SUCCESS {
            let fw = new_address as *mut FileWrapper;
            unsafe {
                (*fw).fs = self.fs;
                (*fw).fs_wrapper = self;
                (*fw).root = false;
                (*fw).proto.revision = r_efi::protocols::file::REVISION;
                (*fw).proto.open = open;
                (*fw).proto.close = close;
                (*fw).proto.delete = delete;
                (*fw).proto.read = read;
                (*fw).proto.write = write;
                (*fw).proto.get_position = get_position;
                (*fw).proto.set_position = set_position;
                (*fw).proto.get_info = get_info;
                (*fw).proto.set_info = set_info;
                (*fw).proto.flush = flush;
            }

            Some(fw)
        } else {
            None
        }
    }

    fn create_root_file(&self) -> Option<*mut FileWrapper> {
        let size = core::mem::size_of::<FileWrapper>();
        let (status, new_address) = super::ALLOCATOR.lock().allocate_pages(
            AllocateType::AllocateAnyPages,
            MemoryType::LoaderData,
            ((size + super::PAGE_SIZE as usize - 1) / super::PAGE_SIZE as usize) as u64,
            0 as u64,
        );

        // log!("EFI_STUB - root file address: {:x}\n", new_address);

        if status == Status::SUCCESS {
            let fw = new_address as *mut FileWrapper;
            let root_dir = self.fs.root().unwrap();
            let mut entry = DirectoryEntry {
                name: [0; 11],
                file_type: FileType::Directory,
                cluster: root_dir.cluster.unwrap(),
                size: 0,
                long_name: [0; 255],
            };
            unsafe {
                (*fw).fs = self.fs;
                (*fw).fs_wrapper = self;
                (*fw).root = true;
                (*fw).proto.revision = r_efi::protocols::file::REVISION;
                (*fw).proto.open = open;
                (*fw).proto.close = close;
                (*fw).proto.delete = delete;
                (*fw).proto.read = read;
                (*fw).proto.write = write;
                (*fw).proto.get_position = get_position;
                (*fw).proto.set_position = set_position;
                (*fw).proto.get_info = get_info;
                (*fw).proto.set_info = set_info;
                (*fw).proto.flush = flush;
                (*fw).dir_entry = entry;
                (*fw).directory = root_dir;
            }
            Some(fw)
        } else {
            None
        }
    }

    pub fn new(
        fs: &'a crate::fat::Filesystem,
        block_part_id: Option<u32>,
    ) -> FileSystemWrapper<'a> {
        FileSystemWrapper {
            hw: super::HandleWrapper {
                handle_type: super::HandleType::FileSystem,
            },
            fs,
            proto: SimpleFileSystemProtocol {
                revision: r_efi::protocols::simple_file_system::REVISION,
                open_volume: filesystem_open_volume,
            },
            block_part_id,
        }
    }
}