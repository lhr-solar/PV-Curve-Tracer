//! This file opens up and manages communication between the laptop application and the Nucleo.
//! 
//! For now, it works with an Arduino.
//! 
//! # Info
//! * File: communication.rs
//! * Author: Matthew Yu
//! * Organization: UT Solar Vehicles Team
//! * Date Created: 9/2/20
//! * Last Modified: 9/2/20

use std::{
    error,
    path::Path,
    io::Read,
    str,
};
use serialport::{
    posix::TTYPort,
    SerialPortSettings,
};

const MAX_BUF_SIZE:usize = 10000;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// a Port struct contains necessary information to connect with the USB device. It contains the baud rate, port name, and the port object to R/W data.
pub struct Port {
    port: TTYPort,
    port_name: String,
    baud_rate: u32
}

/// open_serial_comm opens up a connection to the USB port where the Nucleo is plugged in.
/// 
/// # Returns
/// 
/// * A port struct on success, an error on failure.
pub fn open_serial_comm() -> Result<Port> {
    let mut ports = serialport::available_ports().expect("No ports found!");
    if ports.len() == 0 {
        println!("No ports found.");
    } else {
        let port_name = ports.pop().unwrap().port_name;
        let settings:SerialPortSettings = Default::default();

        println!("Opened the first available port at {}", port_name);
        let port = TTYPort::open(Path::new(&port_name), &settings)
            .map_err(|ref e| format!("Port '{}' not available: {}", port_name, e)).unwrap();

        return Ok(Port {
            port: port,
            port_name: String::from(port_name),
            baud_rate: settings.baud_rate
        });
    }
    Err("Unable to open port.".into())
}

/// receive_message attempts to grab a message from the USB device.
/// 
/// # Arguments
/// 
/// * `port` - Port to grab data from
/// 
/// # Returns
/// 
/// * A string on success, an error on failure.
pub fn receive_message(port: &mut Port) -> Result<String> {
    println!("Reading from {} at {} baud at 1Hz", port.port_name, port.baud_rate);

    let mut serial_buf: Vec<u8> = vec![0; MAX_BUF_SIZE];
    match port.port.read(serial_buf.as_mut_slice()) {
        Ok(_res) => Ok(String::from(str::from_utf8(&serial_buf).unwrap())),
        Err(err) => Err(err.into())
    }
}