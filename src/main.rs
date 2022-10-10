use anyhow::{Result, Context};
use config::Config;
use clap::Parser;

use crate::litra::{find_device_path, light_off, light_on, set_brightness, set_temperature};
use crate::models::{Cli, Commands, LitraConfig};

mod models;
mod litra;


fn main() -> Result<()> {

    let config: LitraConfig = Config::builder()
        .add_source(
            config::Environment::with_prefix("LITRA")
        )
        .build()
        .context("Reading configuration")
        .and_then(|cfg| cfg.try_deserialize().context("Deserialize configuration"))
        .context("Parsing configuration")?;

    println!("Configuration: {:?}", config);

    let cli : Cli = Cli::parse();

    if let Some(path) = cli.path.as_deref() {
        println!("USB Device path = {}", path)
    }

    match &cli.command {
        Some(Commands::Init) => {
            let path = find_device_path().context("Find device path")?;
            println!("Path : {}", path);
            println!("Tip: set the environment variable LITRA_PATH to this value to avoid enumeration of devices.");
            Ok(())
        },
        Some(Commands::On) => {
            light_on(&config)
        },
        Some(Commands::Off) => {
            light_off(&config)
        },
        Some(Commands::Brightness {percent}) => {
            set_brightness(&config, *percent)
        },
        Some(Commands::Temperature {temperature}) => {
            set_temperature(&config, *temperature)
        }
        None => {
            println!("No command given.");
            Ok(())
        }
    }

}
