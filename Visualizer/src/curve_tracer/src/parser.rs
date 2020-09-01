/// File: parser.rs
/// Author: Matthew Yu
/// Organization: UT Solar Vehicles Team
/// Date Created: 8/29/20
/// Last Modified: 8/31/20
/// Description: This file parses either a file or packet data send via UART and fills a struct.
///     This struct is used later for visualization or storage.

use std::{
    error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum PacketCommand {
    START,
    TEST,
}

#[derive(PartialEq)]
pub enum PacketType {
    VOLTAGE,
    CURRENT,
    TEMP,
    IRRAD,
}


pub struct CommandPacket {
    pub packet_id: i32,                 // identifier for the packet
    pub packet_command: PacketCommand,  // contains command type [START/TEST]
    pub packet_params: Vec<f32>         // contains optional command data [voltage start, voltage end, resolution]
}

pub struct DataPacket {
    pub packet_id: i32,                 // corresponds to command_packet id that this data belongs to
    pub packet_subid: i32,              // corresponds to which measurement along the test regime the packet belongs to
    pub packet_type: PacketType,        // what measurement type was performed
    pub packet_data: f32                // data
}

pub struct PacketSet {
    pub command_packet: CommandPacket,
    pub data_packets: Vec<DataPacket>,
}

impl CommandPacket {
    /// verify_packet makes sure the internals are valid.
    /// returns true if correct.
    pub fn _verify_packet(&self) -> bool {
        // TODO: this
        false
    }

    /// transmit_packet sends a command packet over USB to the board.
    /// returns true if successfully sent.
    pub fn _transmit_packet(&self) -> bool {
        // TODO: this
        false
    }

    /// receive_packet is an unused method.
    pub fn _receive_packet(&self) -> bool {
        false
    }
}

impl DataPacket {
    /// verify_packet makes sure the internals are valid.
    /// returns true if correct.
    pub fn _verify_packet(&self) -> bool {
        // TODO: this
        false
    }
}

/// parse_buffer attempts to extract a data or command packet from the string.
/// Packet enum if success, error elsewise.
fn parse_buffer(buffer: String) -> Result<(Option<CommandPacket>, Option<DataPacket>)> {
    let args = buffer.split(" ");
    let vec: Vec<&str> = args.collect();
    // command packet
    if vec[0] == "CMD" {
        if vec.len() > 2 {
            // TEST command
            // check for correct parameter types
            if  !vec[1].parse::<i32>().is_ok() || // packet id
                !vec[2].parse::<f32>().is_ok() || // start voltage
                !vec[3].parse::<f32>().is_ok() || // end voltage
                !vec[4].parse::<f32>().is_ok() {  // resolution
                return Err("Invalid packet parameter.".into())
            }
            Ok((
                Some(CommandPacket {
                    packet_id: vec[1].parse::<i32>().unwrap(), 
                    packet_command: PacketCommand::TEST, 
                    packet_params: vec!(
                        vec[2].parse::<f32>().unwrap(),
                        vec[3].parse::<f32>().unwrap(),
                        vec[4].parse::<f32>().unwrap()
                    )
                }), 
                None
            ))
        } else if vec.len() == 2 {
            // START command
            // check for correct parameter types
            if !vec[1].parse::<i32>().is_ok() { // packet id
                return Err("Invalid packet parameter.".into())
            }
            Ok((
                Some(CommandPacket {
                    packet_id: vec[1].parse::<i32>().unwrap(), 
                    packet_command: PacketCommand::START, 
                    packet_params: vec!()
                }), 
                None
            ))
        } else {
            return Err("Invalid parameter list length.".into())
        }
    }

    // data packet
    else if vec[0] == "DATA" {
        // check for correct parameter types
        if  !vec[1].parse::<i32>().is_ok() ||   // packet id
            !vec[2].parse::<i32>().is_ok() ||   // subpacket id
            !vec[3].parse::<i32>().is_ok() ||   // measurement type
            !vec[4].parse::<f32>().is_ok() {    // measurement value
            return Err("Invalid packet parameter.".into())
        }
        // parse packet measurement type
        let packet_type;
        let measurement_type = vec[3].parse::<i32>().unwrap();
        if measurement_type == 0 {
            packet_type = PacketType::VOLTAGE;
        } else if measurement_type == 1 {
            packet_type = PacketType::CURRENT;
        } else if measurement_type == 2 {
            packet_type = PacketType::TEMP;
        } else if measurement_type == 3 {
            packet_type = PacketType::IRRAD;
        } else {
            return Err("Invalid packet type.".into())
        }

        Ok((
            None, 
            Some(DataPacket {
                packet_id: vec[1].parse::<i32>().unwrap(),
                packet_subid: vec[2].parse::<i32>().unwrap(),
                packet_type: packet_type,
                packet_data: vec[4].parse::<f32>().unwrap()
            })
        ))
    }
    // something else - TODO: maybe ignore comments
    else {
        return Err("Invalid packet type.".into())
    }
}

pub fn parse_file(file_path: String) -> Result<Vec<PacketSet>> {
    // check if valid (exists, has correct header, etc)
    if file_path != "exit" {
        if Path::new(&file_path).is_file() {
            let mut f = BufReader::new(File::open(&file_path).unwrap());
            let mut buffer = String::new(); 
            // open and read the first line looking for a valid header
            f.read_line(&mut buffer).unwrap();
            if buffer.trim() == return_header() {
                println!("Matched the header.");
                buffer = String::new();
                let mut packet_sets:Vec<PacketSet> = vec!();
                let mut success = false;
                // then read in the rest, building a set of packet objects
                while let Ok(result) = f.read_line(&mut buffer) {
                    if result != 0 {
                        match parse_buffer(buffer.trim().to_string()) {
                            Ok(res) => {
                                // assume if one works the other won't
                                if let Some(command_packet) = res.0 {
                                    // check to see if ID already exists
                                    let mut found = false;
                                    for packet in &packet_sets {
                                        if packet.command_packet.packet_id == command_packet.packet_id {
                                            found = true;
                                        }
                                    }
                                    if !found {
                                        packet_sets.push(PacketSet {
                                            command_packet: command_packet,
                                            data_packets: vec!()
                                        })
                                    }
                                } else if let Some(data_packet) = res.1 {
                                    // check to see if there is a packet set with packets
                                    for packet in &mut packet_sets {
                                        if packet.command_packet.packet_id == data_packet.packet_id {
                                            packet.data_packets.push(data_packet);
                                            break;
                                        }
                                    }
                                }
                            },
                            Err(err) => println!("{}", err)
                        }
                        buffer = String::new();
                    } else {
                        println!("EOF.");
                        success = true;
                        break;
                    }
                }

                // successful parsing, gather up the packets and return it
                if success {
                    println!("Packets parsed.");
                    return Ok(packet_sets);
                } else {
                    return Err("Packets not successfully parsed.".into());
                }
            } else {
                return Err("Invalid header {}".into());
            }
        } else {
            return Err("Is not a file. Retry.".into());
        }
    } else {
        return Err("Exiting the file selection menu.".into());
    }
}

fn return_header() -> String {
    String::from("Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.")
}