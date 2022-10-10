use clap::{Parser, Subcommand};

#[derive(Debug, Default, serde_derive::Deserialize, PartialEq, Eq)]
pub struct LitraConfig {
    pub path: Option<String>
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name="USB_PATH")]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,
    On,
    Off,
    Brightness {
        #[arg(value_name="PERCENT")]
        percent: u16
    },
    Temperature {
        #[arg(value_name="TEMPERATURE")]
        temperature: u16
    }
}


