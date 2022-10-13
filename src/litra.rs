use std::ffi::CString;
use std::time::Duration;
use anyhow::{Result, Context};
use clap::command;
use hidapi::HidApi;
use rusb::{Device, UsbContext};
use crate::LitraConfig;

const SUPPORTED_PRODUCTS: [VidPid; 1] = [VidPid { vendor_id: 0x046d, product_id: 0xC900 }];


#[derive(PartialEq, PartialOrd, Debug)]
struct VidPid {
    vendor_id: u16,
    product_id: u16,
}

impl VidPid {
    pub fn new(vendor_id: u16, product_id: u16) -> VidPid {
        VidPid { vendor_id, product_id }
    }
}

fn is_litra_device_rusb<T: UsbContext>(device: &Device<T>) -> bool {
    let descriptor = device.device_descriptor().unwrap();
    let vid_pid = VidPid::new(descriptor.vendor_id(), descriptor.product_id());

    SUPPORTED_PRODUCTS.contains(&vid_pid)
}

pub fn find_device_path() -> Result<String> {
    rusb::set_log_level(rusb::LogLevel::Info);
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
                        println!("        endpoint address {} number {} direction {:?} type {:?}", endpoint.address(), endpoint.number(), endpoint.direction(), endpoint.transfer_type())
                    }
                }
            }
        }
    }
    Ok("foo".to_string())
}

const BUF_LEN:usize =20;

fn send_buffer_rusb(config: &LitraConfig, buf: &mut [u8; BUF_LEN]) -> Result<()> {

    let timeout = Duration::from_millis(1000);

    for device in rusb::devices()?.iter().filter(is_litra_device_rusb) {
        let mut handle = device.open().context("Open handle to USB device")?;

        handle.claim_interface(0).context("Claim interface")?;
        //handle.reset().context("Resetting device")?;
        //handle.clear_halt(0x02).context("Clearing halt")?;
        if device.active_config_descriptor().is_err() {
            handle.set_active_configuration(1).context("set active configuration")?;
        }
        handle.set_alternate_setting(0, 0).context("Set alternate setting")?;


        let languages = handle.read_languages(timeout)?;

        println!("Active configuration: {}", handle.active_configuration()?);
        println!("Languages: {:?}", languages);

        //let _ = handle.set_auto_detach_kernel_driver(true);

        let dev_desc = device.device_descriptor()?;

        for language in handle.read_languages(timeout)? {
            println!("language {:?}", language);
            println!("manufacturer: {}", handle.read_manufacturer_string(language, &dev_desc, timeout)?);
            println!("product: {}", handle.read_product_string(language, &dev_desc, timeout)?);
            println!("serial_number: {}", handle.read_serial_number_string(language, &dev_desc, timeout)?);
            println!("configurations: {}", dev_desc.num_configurations());

            for cfg in 1..=dev_desc.num_configurations() {

                //let cfg_desc = device.config_descriptor(cfg).context("get config descriptor")?;
                let cfg_desc = device.active_config_descriptor().context("get config descriptor")?;
                //println!("  configuration: {} - {}", cfg_desc.number(), handle.read_configuration_string(language, &cfg_desc, timeout).context("read config string")?);
                for intf in cfg_desc.interfaces() {
                    println!("    interface: {}", intf.number());
                    for intf_desc in intf.descriptors() {

                        //println!("      interface: {}", handle.read_interface_string(language, &intf_desc, timeout)?);
                        println!("      number: {}", intf_desc.interface_number());
                        println!("      setting: {}", intf_desc.setting_number());
                        println!("      class: {}", intf_desc.class_code());
                        println!("      sub_class_code: {}", intf_desc.sub_class_code());
                        println!("      protocol_code: {}", intf_desc.protocol_code());
                        println!("      num_endpoints: {}", intf_desc.num_endpoints());
                        for endpoint in intf_desc.endpoint_descriptors() {
                            println!("        endpoint: {}", endpoint.number());
                            println!("          address: {}", endpoint.address());
                            println!("          direction: {:?}", endpoint.direction());
                            println!("          transfer_type: {:?}", endpoint.transfer_type());
                            println!("          sync_type: {:?}", endpoint.sync_type());
                            println!("          usage_type: {:?}", endpoint.usage_type());
                            println!("          max_packet_size: {:?}", endpoint.max_packet_size());
                            println!("          interval: {:?}", endpoint.interval());
                        }
                    }
                }
            }

        }

        match handle.write_interrupt(0x02, buf, timeout) {
            Ok(n) => {println!("Wrote {} bytes", n)}
            Err(err) => {println!("Error {:?}", err)}
        };
        match handle.read_interrupt(0x82, buf, timeout) {
            Ok(n) => {println!("Read {} bytes", n)}
            Err(err) => {println!("Error {:?}", err)}
        };
        handle.release_interface(0).context("Release interface")?;
    }
    Ok(())

}

fn send_buffer(config: &LitraConfig, buf: &mut [u8; BUF_LEN]) -> Result<()> {
    let api = HidApi::new().context("Creating HidApi.")?;
    for device in api.device_list()
        .filter(|dev|
            dev.product_id() == config.product_id
            && dev.vendor_id() == config.vendor_id
        ) {
        println!("Sending data to {:?}", device.manufacturer_string());
        println!("path: {:?}", device.path());
        println!("interface: {}", device.interface_number());
        println!("serial number: {:?}", device.serial_number());
        println!("usage page: {:?}", device.usage_page());
        println!("usage: {:?}", device.usage());
        let device = device.open_device(&api).context("Opening device")?;
        match device.write(buf) {
            Ok(n) => println!("Wrote {} bytes", n),
            Err(err) => println!("Error: {:?}", err)
        }
    }

    Ok(())
}

fn send_command(config: &LitraConfig, command: u8, argument: u16) -> Result<()> {
    let mut buf = [0; BUF_LEN];
    let low_byte: u8 = (argument & 0xff) as u8;
    let high_byte: u8 = (argument >> 8 & 0xff) as u8;
    let msg = [0x11, 0xff, 0x04, command, high_byte, low_byte];
    buf[..msg.len()].copy_from_slice(&msg);

    send_buffer(config, &mut buf).context("Sending HID Data")
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

pub fn set_brightness(config: &LitraConfig, percent: u16) -> Result<()> {
    let brightness = MIN_BRIGHTNESS + (MAX_BRIGHTNESS - MIN_BRIGHTNESS) * percent / 100;
    send_command(config, 0x4c, brightness)
}

pub fn set_temperature(config: &LitraConfig, temperature: u16) -> Result<()> {
    send_command(config, 0x9c, temperature)
}

