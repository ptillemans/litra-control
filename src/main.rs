use std::ffi::CString;
use std::time::Duration;
use anyhow::{Result, Context};
use hidapi::{HidApi, DeviceInfo, HidDevice};
use clap::{Parser, Subcommand};

#[derive(PartialEq, PartialOrd, Debug)]
struct VidPid {
    vendor_id: u16,
    product_id: u16
}

impl VidPid {

    pub fn new(vendor_id: u16, product_id: u16) -> VidPid {
        VidPid{vendor_id, product_id}
    }

}


const SUPPORTED_PRODUCTS: [VidPid; 1] = [VidPid{ vendor_id: 0x046d, product_id: 0xC900} ];


fn is_litra_device(device: &DeviceInfo) -> bool {
    let vid_pid = VidPid::new(device.vendor_id(), device.product_id());

    SUPPORTED_PRODUCTS.contains(&vid_pid)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name="USB_PATH")]
    path: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    On,
    Off,
}



fn find_device_path() -> anyhow::Result<()>{
    Ok(())
}

fn light_on(device: &HidDevice) -> anyhow::Result<()> {
    println!("Turn Litra on.");
    device.write(&[0x11, 0xff, 0x04, 0x1e, 0x01])
        .map(|_| ())
        .context("Turning light on")
}

fn light_off(device: &HidDevice) -> anyhow::Result<()> {
    println!("Turn Litra off.");
    device.write(&[0x11, 0xff, 0x04, 0x1e, 0x00])
        .map(|_| ())
        .context("Turning light off")
}


fn main() -> anyhow::Result<()> {

    let cli = Cli::parse();

    if let Some(path) = cli.path.as_deref() {
        println!("USB Device path = {}", path)
    }

    let path = "\\\\?\\HID#VID_046D&PID_C900&Col02#a&8fac6bd&0&0001#{4d1e55b2-f16f-11cf-88cb-001111000030}";
    let api = HidApi::new_without_enumerate().context("Creating HidApi.")?;
    let device = api.open_path(&CString::new(path).unwrap()).context("Opening connection to Litra")?;

    match &cli.command {
        Some(Commands::Init) => {
            find_device_path()
        },
        Some(Commands::On) => {
            light_on(&device)
        },
        Some(Commands::Off) => {
            light_off(&device)
        },
        None => {
            println!("No command given.");
            Ok(())
        }
    }

            // api.refresh_devices().unwrap();
            // let devices = api.device_list().filter(|dev| is_litra_device(dev));
            //
            // for device in devices {
            //    println!("{:04x}:{:04x} {:?} -> {:?}", device.vendor_id(), device.product_id(),device.product_string(), device.path());
}
