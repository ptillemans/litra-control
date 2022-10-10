use std::ffi::CString;
use hidapi::{DeviceInfo, HidApi};
use anyhow::{Result, Context};
use crate::LitraConfig;

const SUPPORTED_PRODUCTS: [VidPid; 1] = [VidPid{ vendor_id: 0x046d, product_id: 0xC900} ];


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

fn is_litra_device(device: &DeviceInfo) -> bool {
    let vid_pid = VidPid::new(device.vendor_id(), device.product_id());

    println!("Checking device {:?}", vid_pid );
    SUPPORTED_PRODUCTS.contains(&vid_pid)
}

pub fn find_device_path() -> Result<String>{

    let api = HidApi::new().context("Creating HidApi.")?;
    for device in api.device_list() {
        println!("{:04x}:{:04x} {:?} -> {:?}",
                 device.vendor_id(), device.product_id(),
                 device.product_string(), device.path());
    }
    let devices = api
        .device_list()
        .filter(|dev| is_litra_device(dev));


    // experience shows it is the second device
    // TODO: find better way to find the device(s) we need as this will not work with
    // multiple lamps
    let device = devices.last().context("Getting last litra device")?;
    device.path().to_str().context("Converting Litra device path")
        .map(|s| String::from(s))
}

fn send_command(config: &LitraConfig, command: u8, argument: u16) -> Result<()> {
    //let mut api = HidApi::new_without_enumerate().context("Creating HidApi.")?;
    let api = HidApi::new_without_enumerate().context("Creating HidApi.")?;
    let path = (config.path.clone()).context("Need USB path to send commands")?;
    let hid_path = CString::new(path).context("Convert path for FFI call")?;
    let device = api.open_path(&hid_path)
        .context("Opening connection to Litra")?;

    let low_byte:u8  = (argument & 0xff) as u8;
    let high_byte:u8  = (argument >> 8 & 0xff) as u8;
    let buf = [0x11, 0xff, 0x04, command, high_byte, low_byte];
    device.write(&buf)
        .map(|_| ())
        .context(format!("Sending command {:02x}", command))
}

pub fn light_on(config: &LitraConfig) -> Result<()> {
    println!("Turn Litra on.");
    send_command(config, 0x1c, 0x0100)
        .context("Turning light on")
}

pub fn light_off(config: &LitraConfig) -> Result<()> {
    println!("Turn Litra off.");
    send_command(config, 0x1c, 0x0100)
        .context("Turning light off")
}

const MIN_BRIGHTNESS: u16 = 0x0;
const MAX_BRIGHTNESS: u16 = 0xff;
pub fn set_brightness(config: &LitraConfig, percent: u16) -> Result<()>{
    let brightness = MIN_BRIGHTNESS + (MAX_BRIGHTNESS - MIN_BRIGHTNESS) * percent / 100;
    send_command(config, 0x4c, brightness)
}

pub fn set_temperature(config: &LitraConfig, temperature: u16) -> Result<()> {
    send_command(config, 0x9c, temperature)
}

