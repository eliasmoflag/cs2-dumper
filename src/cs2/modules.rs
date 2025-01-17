use std::{fs::{create_dir_all, File}, io::{self, Write}, path::Path};
use chrono::{DateTime, Datelike};
use pelite::image::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, IMAGE_OPTIONAL_HEADER64, IMAGE_SECTION_HEADER};
use crate::{config::Config, error::Error, process::{ProcessModule, ProcessTrait}};

pub fn dump(process: &impl ProcessTrait, config: &Config) {
    if let Some(modules) = &config.modules {
        for module_name in modules {
            match process.mod_find(&module_name) {
                Ok(module) => {
                    println!("found module: {} at 0x{:X}", &module_name, module.module_base);
                    
                    if let Err(err) = dump_module(process, &module_name, &module) {
                        println!("failed to dump module: {}, error: {}", &module_name, err);
                    }
                },
                Err(err) => {
                    println!("couldn't find module: {}, error: {}", &module_name, err);
                }
            }
        }
    }
}

fn dump_module(process: &impl ProcessTrait, module_name: &str, module: &ProcessModule)
    -> Result<(), Error> {

    unsafe {
        let mut data = Vec::new();
        data.resize(module.module_size, 0u8);
    
        process.mem_read(module.module_base, &mut data)?;
        
        let dos_header = data.as_mut_ptr() as *mut IMAGE_DOS_HEADER;
        let nt_headers = data.as_mut_ptr().byte_offset((*dos_header).e_lfanew as isize) as *mut IMAGE_NT_HEADERS64;
    
        (*nt_headers).OptionalHeader.ImageBase = module.module_base as u64;
    
        for i in 0..(*nt_headers).FileHeader.NumberOfSections {
            let section = (nt_headers
                .byte_offset((std::mem::size_of::<IMAGE_NT_HEADERS64>() - std::mem::size_of::<IMAGE_OPTIONAL_HEADER64>()) as isize)
                .byte_offset((*nt_headers).FileHeader.SizeOfOptionalHeader as isize) as *mut IMAGE_SECTION_HEADER).offset(i as isize);
    
            (*section).PointerToRawData = (*section).VirtualAddress;
            (*section).SizeOfRawData = (*section).VirtualSize;
        }
    
        let timestamp = DateTime::from_timestamp((*nt_headers).FileHeader.TimeDateStamp.into(), 0).unwrap();
    
        println!(" - timestamp: {}", timestamp.to_string());
    
        let mut filepath = Path::new(&format!("output\\modules\\{}", module_name)).with_extension("");
        create_dir_all(&filepath)?;
    
        filepath.push(format!("{}_{:02}_{:02}_{:04}.{}",
            Path::new(module_name).with_extension("").to_string_lossy(),
            timestamp.day(), timestamp.month(), timestamp.year(),
            &Path::new(module_name).extension().unwrap().to_string_lossy()));
    
        match File::options().create_new(true).write(true).truncate(true).open(&filepath) {
            Ok(mut file) => {
                file.write(&data)?;
            },
            Err(err) => {
                if err.kind() != io::ErrorKind::AlreadyExists {
                    return Err(err.into());
                }
            }
        }
    
        Ok(())
    }
}
