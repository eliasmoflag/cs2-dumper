#![allow(dead_code, non_camel_case_types)]
#![cfg(target_os = "linux")]

use std::{ffi::c_uchar, fs::{self, File}, os::unix::fs::FileExt, path::Path};
use crate::error::Error;
use super::{ProcessModule, ProcessTrait};

pub struct Process {
    process_id: u32,
    mem: Option<File>
}

impl Process {
    pub fn new(process_id: u32) -> Self {
        Self {
            process_id,
            mem: None
        }
    }

    pub fn find_process_by_name(process_name: &str) -> Result<Self, Error> {

        for dir in fs::read_dir("/proc")? {
            let dir = dir?;

            let process_id = dir.file_name();
            let process_id = match process_id.to_str() {
                Some(process_id) => process_id,
                None => continue
            };

            let process_id = match u32::from_str_radix(process_id, 10) {
                Ok(process_id) => process_id,
                Err(_) => continue
            };

            let cmdline = match fs::read(dir.path().join("cmdline")) {
                Ok(cmdline) => cmdline,
                Err(_) => continue
            };
            
            let file_path = &cmdline[0..cmdline.iter()
                .position(|c| *c == 0)
                .unwrap_or(cmdline.len())];

            let file_path = match std::str::from_utf8(unsafe {
                std::slice::from_raw_parts::<u8>(file_path.as_ptr() as *const u8, file_path.len())
            }) {
                Ok(path) => path,
                Err(_) => continue
            };
            
            if Path::new(&file_path).ends_with(process_name) {
                return Ok(Self {
                    process_id,
                    mem: None
                });
            }
        }

        Err(Error::NotFound)
    }
}

impl ProcessTrait for Process {
    fn attach(&mut self) -> Result<(), Error> {
        self.mem = Some(File::options().read(true).open(format!("/proc/{}/mem", self.process_id))?);
        Ok(())
    }

    fn detach(&mut self) -> Result<(), Error> {
        self.mem = None;
        Ok(())
    }

    fn mem_read(&self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        let mem = match &self.mem {
            Some(mem) => mem,
            None => return Err(Error::NotAttached)
        };

        mem.read_at(data, address as u64)?;
        Ok(())
    }

    fn mem_write(&self, address: usize, data: &[u8]) -> Result<(), Error> {
        let mem = match &self.mem {
            Some(mem) => mem,
            None => return Err(Error::NotAttached)
        };

        mem.write_at(data, address as u64)?;
        Ok(())
    }

    fn mod_find(&self, name: &str) -> Result<ProcessModule, Error> {

        let maps = fs::read_to_string(format!("/proc/{}/maps", self.process_id))?;

        let mut found: Vec<((usize, usize), String)> = Vec::new();

        for map in maps.split('\n') {
            let map = map.splitn(6, ' ')
                .map(|s| s.trim_start()).collect::<Vec<&str>>().iter()
                .map(|&s| s.to_string()).collect::<Vec<String>>();

            if map.len() != 6 {
                continue;
            }
            
            let file_path = Path::new(&map[5]);
            let file_name = match file_path.file_name() {
                Some(file_name) => match file_name.to_str() {
                    Some(file_name) => match file_name.to_lowercase().split_terminator('.').next() {
                        Some(file_name) => file_name.to_string(),
                        None => continue
                    },
                    None => continue
                },
                None => continue
            };

            let address_range: Vec<&str> = map[0].splitn(2, '-').collect();
            if address_range.len() != 2 {
                continue;
            }

            let address_range = (
                match usize::from_str_radix(address_range[0], 16) {
                    Ok(begin) => begin,
                    Err(_) => continue
                },
                match usize::from_str_radix(address_range[1], 16) {
                    Ok(end) => end,
                    Err(_) => continue
                },
            );

            if Path::new(&file_name) == Path::new(&name.to_lowercase()).with_extension("") {
                found.push((address_range, file_name));
            }
        }
        
        if found.is_empty() {
            return Err(Error::NotFound);
        }
        
        Ok(ProcessModule {
            module_base: found.first().unwrap().0.0,
            module_size: found.last().unwrap().0.1 - found.first().unwrap().0.0
        })
    }
}

pub const ELFMAG0: u8 = 0x7f;
pub const ELFMAG1: u8 = 0x45;
pub const ELFMAG2: u8 = 0x4c;
pub const ELFMAG3: u8 = 0x46;
pub const ELFMAGIC: [u8; 4] = [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3];

pub type Elf64_Half = u16;
pub type Elf64_Addr = u64;
pub type Elf64_Off = u64;
pub type Elf64_Word = u32;
pub type Elf64_Xword = u64;

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Ehdr {

    pub e_ident: [c_uchar; 16],
    pub e_type: Elf64_Half,
    pub e_machine: Elf64_Half,
    pub e_version: Elf64_Word,
    pub e_entry: Elf64_Addr,
    pub e_phoff: Elf64_Off,
    pub e_shoff: Elf64_Off,
    pub e_flags: Elf64_Word,
    pub e_ehsize: Elf64_Half,
    pub e_phentsize: Elf64_Half,
    pub e_phnum: Elf64_Half,
    pub e_shentsize: Elf64_Half,
    pub e_shnum: Elf64_Half,
    pub e_shstrndx: Elf64_Half,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Shdr {
    pub sh_name: Elf64_Word,
    pub sh_type: Elf64_Word,
    pub sh_flags: Elf64_Xword,
    pub sh_addr: Elf64_Addr,
    pub sh_offset: Elf64_Off,
    pub sh_size: Elf64_Xword,
    pub sh_link: Elf64_Word,
    pub sh_info: Elf64_Word,
    pub sh_addralign: Elf64_Xword,
    pub sh_entsize: Elf64_Xword,
}
