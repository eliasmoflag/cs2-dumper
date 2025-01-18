mod error;
mod config;
mod cs2;
mod platform;

use config::Config;
use error::Error;
use platform::ProcessTrait;

fn main() -> Result<(), Error> {
    
    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => {
            let config = Config::new();
            config.save().ok();
            config
        }
    };

    let mut process = match platform::Process::find_process_by_name(platform::PROCESS_NAME) {
        Ok(process) => process,
        Err(err) => {
            println!("couldn't attach to process: {}, error: {}", platform::PROCESS_NAME, err);
            return Ok(());
        }
    };

    process.attach()?;

    println!("attached to cs2.exe");

    cs2::modules::dump(&mut process, &config);

    Ok(())
}
