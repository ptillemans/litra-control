use std::ffi::CString;
use anyhow::{Result, Context};
use hidapi::HidApi;
use crate::LitraConfig;

pub fn find_device_path(config: &LitraConfig) -> Result<String> {
    HidApi::new().context("Creating HidApi.")?
        .device_list()
        .filter(|dev|
            dev.product_id() == config.product_id
                && dev.vendor_id() == config.vendor_id
        )
        .map(|device| {
            device.path().to_str().unwrap().to_string().clone()
        })
        .next()
        .context("Fetching Litra devices.")
}

const BUF_LEN: usize = 20;

fn send_buffer(config: &LitraConfig, buf: &mut [u8; BUF_LEN]) -> Result<()> {
    let api = HidApi::new().context("Creating HidApi.")?;
    let path = CString::new(config.path.clone().into_bytes())?;

    for device in api.device_list()
        .filter(|dev| dev.vendor_id() == config.vendor_id && dev.product_id() == config.product_id) {
        println!("turning on device {:?}", device.path());
        let device = device.open_device(&api)?;
        let _ = device
            .write(buf).context("writing buffer")
            .map(|n| println!("Wrote {} bytes", n));
    };
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

