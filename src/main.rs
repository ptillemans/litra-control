use std::time::Duration;
use anyhow::{Context, Result};
use rusb::{UsbContext, Device, DeviceHandle, devices};

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


fn is_litra_device<T:UsbContext>(device: &Device<T>) -> bool {
    let descriptor = device.device_descriptor().unwrap();
    let vid_pid = VidPid::new(descriptor.vendor_id(), descriptor.product_id());

    SUPPORTED_PRODUCTS.contains(&vid_pid)
}

fn main() {
    rusb::set_log_level(rusb::LogLevel::Debug);
    for device in rusb::devices().unwrap().iter()
        .filter(is_litra_device) {

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


        let mut handle = device.open().unwrap();
        let timeout = Duration::from_millis(1000);
        handle.set_active_configuration(1).unwrap();
        handle.claim_interface(0).unwrap();
        handle.set_alternate_setting(0, 0).unwrap();
        let mut buf = [0; 20];
        let msg = [0x11, 0xff, 0x04, 0x1e, 0x01];
        buf[..msg.len()].copy_from_slice(&msg);
        match handle.write_interrupt(2, &buf, timeout) {
            Ok(len) => {
                println!("Wrote {} bytes to the Litra device", len);
            }
            Err(err) => {
                println!("Could not write to endpoint: {:?}", err)
            }
        }
        handle.release_interface(0);

    }
}
