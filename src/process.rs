#![allow(dead_code)]

use std::{ffi, path::Path};
use windows::Win32::{
    Foundation::{
        CloseHandle,
        ERROR_NO_MORE_FILES, HANDLE
    },
    System::{
        Diagnostics::{
            Debug::{
                ReadProcessMemory,
                WriteProcessMemory
            },
            ToolHelp::{
                CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next, MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS
            }
        },
        Threading::{
            OpenProcess,
            PROCESS_ALL_ACCESS
        }
    }
};

use crate::error::Error;

pub trait ProcessTrait {
    fn attach(&mut self) -> Result<(), Error>;
    fn detach(&mut self) -> Result<(), Error>;
    
    fn mem_read(&self, address: usize, data: &mut [u8]) -> Result<(), Error>;
    fn mem_write(&self, address: usize, data: &[u8]) -> Result<(), Error>;

    fn mod_find(&self, name: &str) -> Result<ProcessModule, Error>;
}

pub struct ProcessModule {
    pub module_base: usize,
    pub module_size: usize
}

#[derive(Clone)]
pub struct WindowsProcess {
    process_id: u32,
    process_handle: Option<HANDLE>
}

impl WindowsProcess {
    pub fn new(process_id: u32) -> Self {
        Self {
            process_id,
            process_handle: None
        }
    }

    pub fn find_process_by_name(process_name: &str) -> Result<Self, Error> {

        let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
            Ok(handle) => handle,
            Err(error) => return Err(Error::WindowsError(error))
        };
    
        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
    
        if let Err(error) = unsafe { Process32First(snapshot, &mut entry) } {
            return Err(Error::WindowsError(error));
        }
    
        let mut process: Option<Self> = None;
        loop {
            let filepath = &entry.szExeFile[0..entry.szExeFile.iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len())];
    
            let filepath = match std::str::from_utf8(unsafe {
                std::slice::from_raw_parts::<u8>(filepath.as_ptr() as *const u8, filepath.len())
            }) {
                Ok(path) => path,
                Err(error) => return Err(Error::Utf8Error(error))
            };

            if filepath == process_name {
                process = Some(Self::new(entry.th32ProcessID));
                break;
            }
            
            match unsafe { Process32Next(snapshot, &mut entry) } {
                Ok(_) => (),
                Err(error) => {
                    if error == ERROR_NO_MORE_FILES.into() {
                        break;
                    }
                    return Err(Error::WindowsError(error));
                }
            }
        }

        unsafe { CloseHandle(snapshot).ok() };

        match process {
            Some(process) => {
                Ok(process)
            }
            None => Err(Error::NotFound)
        }
    }    
}

impl ProcessTrait for WindowsProcess {
    fn attach(&mut self) -> Result<(), Error> {
        if self.process_handle.is_some() {
            return Err(Error::AlreadyAttached)
        }

        Ok(match unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, self.process_id) } {
            Ok(handle) => self.process_handle = Some(handle),
            Err(error) => return Err(Error::WindowsError(error))
        })
    }

    fn detach(&mut self) -> Result<(), Error> {
        let handle = match self.process_handle {
            Some(handle) => handle,
            None => return Err(Error::NotAttached)
        };

        Ok(match unsafe { CloseHandle(handle) } {
            Ok(_) => self.process_handle = None,
            Err(error) => return Err(Error::WindowsError(error))
        })
    }

    fn mem_read(&self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        let handle = match self.process_handle {
            Some(handle) => handle,
            None => return Err(Error::NotAttached)
        };

        Ok(if let Err(error) = unsafe { ReadProcessMemory(
            handle,
            address as _,
            data.as_mut_ptr() as *mut ffi::c_void,
            data.len(),
            None) } {

            return Err(Error::WindowsError(error));
        })
    }

    fn mem_write(&self, address: usize, data: &[u8]) -> Result<(), Error> {
        let handle = match self.process_handle {
            Some(handle) => handle,
            None => return Err(Error::NotAttached)
        };
        
        Ok(if let Err(error) = unsafe { WriteProcessMemory(
            handle,
            address as _,
            data.as_ptr() as _,
            data.len(),
            None) } {

            return Err(Error::WindowsError(error));
        })
    }

    fn mod_find(&self, name: &str) -> Result<ProcessModule, Error> {

        let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, self.process_id) } {
            Ok(handle) => handle,
            Err(error) => return Err(Error::WindowsError(error))
        };
    
        let mut entry = MODULEENTRY32::default();
        entry.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;
    
        if let Err(error) = unsafe { Module32First(snapshot, &mut entry) } {
            return Err(Error::WindowsError(error));
        }
    
        let mut module: Option<ProcessModule> = None;
        loop {
            let filepath = &entry.szExePath[0..entry.szExePath.iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExePath.len())];
    
            let filepath = match std::str::from_utf8(unsafe {
                std::slice::from_raw_parts::<u8>(filepath.as_ptr() as *const u8, filepath.len())
            }) {
                Ok(path) => path,
                Err(error) => return Err(Error::Utf8Error(error))
            };
            
            if let Some(filepath) = Path::new(&filepath.to_lowercase()).with_extension("").file_name() {
                if filepath == Path::new(&name.to_lowercase()).with_extension("") {
                    module = Some(ProcessModule {
                        module_base: entry.modBaseAddr as usize,
                        module_size: entry.modBaseSize as usize
                    });
                    break;
                }
            }
            else {
                if Path::new(filepath) == Path::new(&name.to_lowercase()).with_extension("") {
                    module = Some(ProcessModule {
                        module_base: entry.modBaseAddr as usize,
                        module_size: entry.modBaseSize as usize
                    });
                    break;
                }
            }
            
            match unsafe { Module32Next(snapshot, &mut entry) } {
                Ok(_) => (),
                Err(error) => {
                    if error == ERROR_NO_MORE_FILES.into() {
                        break;
                    }
                    return Err(Error::WindowsError(error));
                }
            }
        }

        unsafe { CloseHandle(snapshot).ok() };

        match module {
            Some(module) => {
                Ok(module)
            }
            None => Err(Error::NotFound)
        }
    }
}
