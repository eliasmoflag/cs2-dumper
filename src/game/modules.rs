use crate::{config::Config, error::{Error, Result}, platform::{ProcessModule, ProcessTrait}};
use std::{ffi::OsStr, fs::{create_dir_all, File}, io::{self, Write}, path::Path};
use chrono::{DateTime, Datelike, Utc};

#[cfg(target_os = "windows")]
use pelite::image::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, IMAGE_OPTIONAL_HEADER64, IMAGE_SECTION_HEADER};

pub fn dump(process: &impl ProcessTrait, config: &Config) {
    let modules = match &config.modules {
        Some(modules) => modules,
        None => return
    };

    for mod_name in modules {
        let module = match process.mod_find(mod_name) {
            Ok(module) => module,
            Err(err) => {
                println!("failed to find module: {}, error: {}", mod_name, err);
                continue
            }
        };

        match dump_module(process, mod_name, &module) {
            Ok(_) => println!("dumped module: {} at 0x{:X}", mod_name, module.module_base),
            Err(err) => {
                if let Error::IoError(err) = &err {
                    if err.kind() == io::ErrorKind::AlreadyExists {
                        println!("module already dumped: {}", mod_name);
                    }
                } else {
                    println!("failed to dump module: {}, error: {}", mod_name, err);
                }
            },
        }
    }
}

fn write_dump(name: &str, timestamp: DateTime<Utc>, data: &[u8]) -> Result<()> {
    let mut file_path = Path::new(&format!("output/modules/{}", name)).with_extension("");

    create_dir_all(&file_path)?;

    let file_name = Path::new(name).with_extension("");
    let file_name = file_name.file_name().unwrap().to_str().unwrap();
    let mut file_extension = String::from(Path::new(name).extension().unwrap_or(OsStr::new("")).to_str().unwrap());
    if !file_extension.is_empty() {
        file_extension.insert(0, '.');
    }

    let file_name = format!("{}_{:02}_{:02}_{:04}{}",
        file_name, timestamp.day(), timestamp.month(), timestamp.year(), file_extension
    );

    file_path.push(file_name);

    match File::options().create_new(true).write(true).open(&file_path) {
        Ok(mut file) => {
            file.write(&data)?;
        },
        Err(err) => {
            return Err(err.into());
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn get_timestamp(data: &[u8]) -> Result<DateTime<Utc>> {
    let dos_header = data.as_ptr() as *mut IMAGE_DOS_HEADER;
    let nt_headers = data.as_ptr().byte_offset((*dos_header).e_lfanew as isize) as *mut IMAGE_NT_HEADERS64;
    Ok(DateTime::from_timestamp((*nt_headers).FileHeader.TimeDateStamp.into(), 0).unwrap())
}

#[cfg(target_os = "linux")]
fn get_timestamp(data: &[u8]) -> Result<DateTime<Utc>> {
    Ok(Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc())
}

fn dump_module(process: &impl ProcessTrait, module_name: &str, module: &ProcessModule)
    -> Result<()> {
    
    let mut data: Vec<u8> = Vec::new();
    data.resize(module.module_size, b'\0');

    process.mem_read(module.module_base, &mut data)?;
    
    unsafe {
        fix_image(module.module_base, &mut data)?;
    
        let timestamp = get_timestamp(&data)?;
    
        write_dump(module_name, timestamp, &data)?;
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn fix_image(allocation_base: usize, data: &mut [u8]) -> Result<()> {

    let dos_header = data.as_mut_ptr() as *mut IMAGE_DOS_HEADER;
    let nt_headers = data.as_mut_ptr().byte_offset((*dos_header).e_lfanew as isize) as *mut IMAGE_NT_HEADERS64;

    (*nt_headers).OptionalHeader.ImageBase = allocation_base as u64;

    for i in 0..(*nt_headers).FileHeader.NumberOfSections {
        let section = (nt_headers
            .byte_offset((std::mem::size_of::<IMAGE_NT_HEADERS64>() - std::mem::size_of::<IMAGE_OPTIONAL_HEADER64>()) as isize)
            .byte_offset((*nt_headers).FileHeader.SizeOfOptionalHeader as isize) as *mut IMAGE_SECTION_HEADER).offset(i as isize);

        (*section).PointerToRawData = (*section).VirtualAddress;
        (*section).SizeOfRawData = (*section).VirtualSize;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
unsafe fn fix_image(_allocation_base: usize, data: &mut [u8]) -> Result<()> {
    use crate::{error::Error, platform::linux::{Elf64_Ehdr, Elf64_Shdr, ELFMAGIC}};


    if data.len() < std::mem::size_of::<Elf64_Ehdr>() {
        return Err(Error::InvalidImage);
    }

    let ehdr = data.as_mut_ptr() as *mut Elf64_Ehdr;
    
    if (*ehdr).e_ident[0..4] != ELFMAGIC {
        return Err(Error::InvalidImage);
    }

    if (*ehdr).e_shoff == 0 {
        return Ok(());
    }

    for i in 0..(*ehdr).e_shnum {

        let shdr = (ehdr.byte_offset((*ehdr).e_shoff as isize) as *mut Elf64_Shdr).offset(i as isize);
        
        (*shdr).sh_addr = (*shdr).sh_offset;
    }

    Ok(())
}
