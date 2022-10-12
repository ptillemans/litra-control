use std::ffi::CString;
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
    Brightness {
        #[arg(value_name="PERCENT")]
        percent: u16
    },
    Temperature {
        #[arg(value_name="TEMPERATURE")]
        temperature: u16
    }
}



fn find_device_path(api: &mut HidApi) -> Result<String>{

    api.refresh_devices().context("Refreshing devices")?;
    let devices = api.device_list().filter(|dev| is_litra_device(dev));

    //for device in devices {
    //    println!("{:04x}:{:04x} {:?} -> {:?}", device.vendor_id(), device.product_id(), device.product_string(), device.path());
    //}

    // experience shows it is the second device
    // TODO: find better way to find the device(s) we need as this will not work with
    // multiple lamps
    let device = devices.last().context("Getting last litra device")?;
    device.path().to_str().context("Converting Litra device path")
        .map(|s| String::from(s))
}

fn send_command(device: &HidDevice, command: u8, argument: u16) -> Result<()> {
    let low_byte:u8  = (argument & 0xff) as u8;
    let high_byte:u8  = (argument >> 8 & 0xff) as u8;
    let buf = [0x11, 0xff, 0x04, command, high_byte, low_byte];
    device.write(&buf)
        .map(|_| ())
        .context(format!("Sending command {:02x}", command))
}

fn light_on(device: &HidDevice) -> Result<()> {
    println!("Turn Litra on.");
    send_command(device, 0x1c, 0x0100)
        .context("Turning light on")
}

fn light_off(device: &HidDevice) -> Result<()> {
    println!("Turn Litra off.");
    send_command(device, 0x1c, 0x0100)
        .context("Turning light off")
}

const MIN_BRIGHTNESS: u16 = 0x14;
const MAX_BRIGHTNESS: u16 = 0xfa;
fn set_brightness(device: &HidDevice, percent: u16) -> Result<()>{
    let brightness = MIN_BRIGHTNESS + (MAX_BRIGHTNESS - MIN_BRIGHTNESS) * percent / 100;
    send_command(device, 0x4c, brightness)
}

fn set_temperature(device: &HidDevice, temperature: u16) -> Result<()> {
   send_command(device, 0x9c, temperature)
}

fn main() -> Result<()> {

    let cli = Cli::parse();

    if let Some(path) = cli.path.as_deref() {
        println!("USB Device path = {}", path)
    }

    let path = "\\\\?\\HID#VID_046D&PID_C900&Col02#a&8fac6bd&0&0001#{4d1e55b2-f16f-11cf-88cb-001111000030}";
    let mut api = HidApi::new_without_enumerate().context("Creating HidApi.")?;
    let device = api.open_path(&CString::new(path).unwrap()).context("Opening connection to Litra")?;

    match &cli.command {
        Some(Commands::Init) => {
            let path = find_device_path(&mut api).context("Find device path")?;
            println!("Path : {}", path);
            println!("Tip: set the environment variable LITRA_PATH to this value to avoid enumeration of devices.");
            Ok(())
        },
        Some(Commands::On) => {
            light_on(&device)
        },
        Some(Commands::Off) => {
            light_off(&device)
        },
        Some(Commands::Brightness {percent}) => {
            set_brightness(&device, *percent)
        },
        Some(Commands::Temperature {temperature}) => {
            set_temperature(&device, *temperature)
        }
        None => {
            println!("No command given.");
            Ok(())
        }
    }

}
