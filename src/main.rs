use anyhow::Result;
use config::Config;
use clap::Parser;

use crate::litra::{find_device_path, light_off, light_on, set_brightness, set_temperature};
use crate::models::{Cli, Commands, LitraConfig};

mod models;
mod litra;


fn main() -> Result<()> {

    let cli : Cli = Cli::parse();

    let config = Config::builder()
        .set_default("vendor_id", "1133")?
        .set_default("product_id", "51456")?
        .set_default("path", cli.path.unwrap_or("1-4:1.0".to_string()))?
        .add_source(
            config::Environment::with_prefix("LITRA")
        )
        .build()?;
    let config: LitraConfig =
        config.try_deserialize()?;

    println!("Configuration: {:?}", config);

    match &cli.command {
        Some(Commands::Init) => {
            println!("Scanning USB devices. This might take a few seconds.");
            match find_device_path(&config) {
                Ok(paths) => {
                    if paths.len() > 0 {
                        println!();
                        for path in paths {
                            println!("  HID Path : {}", path);
                        }
                        println!();
                        println!("Set the environment variable LITRA_PATH to one of these values to avoid enumeration of devices.");
                        println!("Unfortunately it is hard to specify which one is the right one as it depends on the platform.");
                        println!("On Windows there are 2 per light and the second one is the one you want.");
                    } else {
                        println!();
                        println!("No Litra devices were found.")
                    }
                }
                Err(err) => {
                    println!("Error during searching for Litra devices : {:?}", err)
                }
            }
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
