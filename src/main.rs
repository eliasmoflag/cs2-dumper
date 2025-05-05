mod error;
mod config;
mod game;
mod platform;

use config::Config;
use error::Error;
use platform::ProcessTrait;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    process: String
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let config = match Config::load() {
        Ok(config) => config,
        Err(_) => {
            let config = Config::new();
            config.save().ok();
            config
        }
    };

    let mut process = match platform::Process::find_process_by_name(&args.process) {
        Ok(process) => process,
        Err(err) => {
            println!("couldn't attach to process: {}, error: {}", args.process, err);
            return Ok(());
        }
    };

    process.attach()?;

    println!("attached to {}", args.process);

    game::modules::dump(&mut process, &config);

    Ok(())
}
