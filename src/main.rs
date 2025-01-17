mod error;
mod process;
mod config;
mod cs2;

use config::Config;
use error::Error;
use process::{ProcessTrait, WindowsProcess};

fn main() -> Result<(), Error> {
    
    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => {
            let config = Config::new();
            config.save().ok();
            config
        }
    };
    
    let mut process = match WindowsProcess::find_process_by_name("cs2.exe") {
        Ok(process) => process,
        Err(err) => {
            println!("couldn't attach to process: cs2.exe, error: {:?}", err);
            return Ok(());
        }
    };

    process.attach()?;

    println!("attached to cs2.exe");

    cs2::modules::dump(&mut process, &config);

    Ok(())
}
