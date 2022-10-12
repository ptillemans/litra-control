use std::ffi::CString;
use std::time::Duration;
use anyhow::{Result, Context};
use rusb::{Device, UsbContext};
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

fn is_litra_device_rusb<T:UsbContext>(device: &Device<T>) -> bool {
    let descriptor = device.device_descriptor().unwrap();
    let vid_pid = VidPid::new(descriptor.vendor_id(), descriptor.product_id());

    SUPPORTED_PRODUCTS.contains(&vid_pid)
}

pub fn find_device_path() -> Result<String>{
    rusb::set_log_level(rusb::LogLevel::Debug);
    for device in rusb::devices().unwrap().iter()
        .filter(is_litra_device_rusb) {

        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                 device.bus_number(),
                 device.address(),
                 device_desc.vendor_id(),
                 device_desc.product_id());

        for n in 0..device_desc.num_configurations() as u8 {
            let config_desc = device.config_descriptor(n).unwrap();
            println!("Configuration {} -> {} interfaces ", config_desc.number(), config_desc.num_interfaces());
            for interface in config_desc.interfaces() {
                println!("   Interface {} ", interface.number());
                for if_desc in interface.descriptors() {

                    println!("      number {} endpoints: {} setting number: {}", if_desc.interface_number(), if_desc.num_endpoints(), if_desc.setting_number());

                    for endpoint in if_desc.endpoint_descriptors() {
                        println!("        endpoint address {} number {} direction {:?} type {:?}", endpoint.address(), endpoint.number(), endpoint.direction(), endpoint.transfer_type() )
                    }
                }
            }
        }
    }
    Ok("foo".to_string())
}

fn send_command(config: &LitraConfig, command: u8, argument: u16) -> Result<()> {
    let mut buf = [0; 20];
    let low_byte:u8  = (argument & 0xff) as u8;
    let high_byte:u8  = (argument >> 8 & 0xff) as u8;
    let msg = [0x11, 0xff, 0x04, command, high_byte, low_byte];
    buf[..msg.len()].copy_from_slice(&msg);

    let vid = config.vendor_id;
    let pid = config.product_id;
    let mut handle = rusb::open_device_with_vid_pid(vid, pid).context("Open handle to USB device")?;
    handle.set_auto_detach_kernel_driver(true).context("Set autodetach kernel driver")?;
    let timeout = Duration::from_millis(100);
    handle.reset().context("Resetting device")?;
    handle.claim_interface(0).context("Claim interface")?;
    // handle.set_alternate_setting(0, 0).context("Set alternate setting")?;
    handle.write_interrupt(2, &buf, timeout)
        .context("Writing to device")?;
    handle.read_interrupt(0x82, &mut buf, timeout)
        .context("Reading response from device")?;
    handle.release_interface(0).context("Release interface")
}

pub fn light_on(config: &LitraConfig) -> Result<()> {
    println!("Turn Litra on.");
    send_command(config, 0x1c, 0x0100)
        .context("Turning light on")
}

pub fn light_off(config: &LitraConfig) -> Result<()> {
    println!("Turn Litra off.");
    send_command(config, 0x1c, 0x0000)
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

