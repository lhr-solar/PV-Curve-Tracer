use serialport::{
    posix::TTYPort,
    SerialPortSettings,
};
use std::{
    error,
    path::Path,
    io::{Read, Write},
    str,
};

/// maximum number of characters the serial buffer can read at a time
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
    let ports = serialport::available_ports();
    if let Ok(mut ports) = ports {
        if ports.len() != 0 {
            // grab the first available port
            let port_name = ports.pop().unwrap().port_name;
            let settings:SerialPortSettings = Default::default();
    
            // open a TTY port NOTE: that this only works for linux machines
            println!("[open_serial_comm] Opened the first available port at {}", port_name);
            let port = TTYPort::open(Path::new(&port_name), &settings)
                .map_err(|ref e| format!("[open_serial_comm] Port '{}' not available: {}", port_name, e));
    
            match port {
                Ok(port) => {
                    return Ok(Port {
                        port: port,
                        port_name: String::from(port_name),
                        baud_rate: settings.baud_rate
                    });
                },
                Err(err) => {
                    return Err(format!("{}", err).into());
                }
            }
        }
        return Err("[open_serial_comm] No ports found.".into());
    } 
    Err("[open_serial_comm] Unable to open port.".into())
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
    // println!("[receive_message] Reading from {} at {} baud", port.port_name, port.baud_rate);

    let mut serial_buf: Vec<u8> = vec![0; MAX_BUF_SIZE];
    match port.port.read(serial_buf.as_mut_slice()) {
        Ok(_res) => Ok(String::from(str::from_utf8(&serial_buf).unwrap())),
        Err(err) => Err(err.into())
    }
}

/// send_message attempts to send a command over serial to the Nucleo.
/// 
/// # Arguments
/// 
/// * `port` - Port to grab data from
/// * `message` - message to write to the Nucleo
/// 
/// # Returns
/// 
/// * Nothing on success, an error on failure.
pub fn send_message(port: &mut Port, message: String) -> Result<()> {
    println!("[send_message] Writing \"{}\" to {} at {} baud", message, port.port_name, port.baud_rate);

    match port.port.write(message.as_bytes()) {
        Ok(_res) => Ok(()),
        Err(err) => Err(err.into())
    }
}