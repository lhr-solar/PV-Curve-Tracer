//! This file parses either a file or packet data send via UART and fills a struct. This struct is used later for visualization or storage.
//! 
//! # Info
//! * File: parser.rs
//! * Author: Matthew Yu
//! * Organization: UT Solar Vehicles Team
//! * Date Created: 8/29/20
//! * Last Modified: 9/7/20

use std::{
    error,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};
use chrono::{DateTime, Utc};
use crate::{
    port::*,
    visualizer::*
};

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// the PacketCommand enum is used to differentiate between a start and test command. The Nucleo should not begin
/// listening for TEST commands unless the START command is transmitted.
#[derive(PartialEq)]
pub enum PacketCommand {
    START,
    TEST,
    END
}
impl PacketCommand {
    /// to_num converts a PacketCommand into an i32.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * respective i32 value.
    pub fn _to_num(&self) -> i32 {
        match self {
            PacketCommand::START => 0,
            PacketCommand::TEST => 1,
            PacketCommand::END => 2,
        }
    }

    /// num_to_packet_type converts an i32 into a PacketCommand.
    /// 
    /// # Arguments
    /// 
    /// * `int` - number to convert
    /// 
    /// # Returns
    /// 
    /// * respective PacketCommand enum.
    pub fn _num_to_packet_type(val: i32) -> PacketCommand {
        match val {
            0 => PacketCommand::START,
            1 => PacketCommand::TEST,
            2 => PacketCommand::END,
            _ => PacketCommand::END
        }
    }
}


/// the PacketType enum is used to differentiate between the data returned in the Data Packet.
/// at the moment, only Voltage, Current, and Temperature sensors are supported.
#[derive(PartialEq)]
pub enum PacketType {
    VOLTAGE,
    CURRENT,
    TEMP,
    IRRAD,
}
impl PacketType {
    /// to_num converts a PacketType into an i32.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * respective i32 value.
    pub fn to_num(&self) -> i32 {
        match self {
            PacketType::VOLTAGE => 0,
            PacketType::CURRENT => 1,
            PacketType::TEMP => 2,
            PacketType::IRRAD => 3
        }
    }

    /// num_to_packet_type converts an i32 into a PacketType.
    /// 
    /// # Arguments
    /// 
    /// * `int` - number to convert
    /// 
    /// # Returns
    /// 
    /// * respective PacketType enum.
    pub fn num_to_packet_type(val: i32) -> PacketType {
        match val {
            0 => PacketType::VOLTAGE,
            1 => PacketType::CURRENT,
            2 => PacketType::TEMP,
            3 => PacketType::IRRAD,
            _ => PacketType::VOLTAGE
        }
    }
}



/// the CommandPacket struct is generated when parsing UART or file data and represents a single command.
/// it contains the following:
/// 
/// * packet id, representing its order of execution if many commands are sent.
/// * packet command - the type of command that should be executed by the Nucleo.
/// * packet params - parameters translated by the Nucleo to represent how the test regime should be executed.
pub struct CommandPacket {
    pub packet_id: i32,                 // identifier for the packet
    pub packet_command: PacketCommand,  // contains command type [START/TEST]
    pub packet_params: Vec<f32>         // contains optional command data [voltage start, voltage end, resolution]
}
impl CommandPacket {
    pub fn new(packet_id: i32, packet_command: PacketCommand, packet_params: Vec<f32>) -> CommandPacket {
        CommandPacket {
            packet_id: packet_id,
            packet_command: packet_command,
            packet_params: packet_params
        }
    }

    /// parse_packet_string parses and typechecks a string and converts it into a CommandPacket if applicable.
    /// 
    /// # Arguments
    /// 
    /// * `string` - string to parse and verify
    /// 
    /// # Returns
    /// 
    /// * A CommandPacket on success, an error on failure.
    pub fn parse_packet_string(string: String) -> Result<CommandPacket> {
        let args = string.split(" ");
        let vec: Vec<&str> = args.collect();

        // command packet
        if (vec[0] == "START") || (vec[0] == "END") {
            // check for exactly 2 parameters
            if vec.len() != 2 {
                return Err("[parse_packet_string] Invalid parameter list length.".into());
            }
            // check for valid ID type
            if !vec[1].parse::<i32>().is_ok() {
                return Err("[parse_packet_string] Invalid packet parameter types.".into());
            }
            if vec[0] == "START" {
                // build the CommandPacket
                let command_packet = CommandPacket::new(
                    vec[1].parse::<i32>().unwrap(),
                    PacketCommand::START,
                    vec!()
                );
                // verify it
                match command_packet.verify_packet() {
                    Ok(_) => Ok(command_packet),
                    Err(err) => Err(err)
                }
            } else {
                // build the CommandPacket
                let command_packet = CommandPacket::new(
                    vec[1].parse::<i32>().unwrap(),
                    PacketCommand::END,
                    vec!()
                );
                // verify it
                match command_packet.verify_packet() {
                    Ok(_) => Ok(command_packet),
                    Err(err) => Err(err)
                }
            }
        } else if vec[0] == "TEST" {
            // check for exactly 5 parameters
            if vec.len() != 5 {
                return Err("[parse_packet_string] Invalid parameter list length.".into());
            }
            // check for valid parameter types
            if  !vec[1].parse::<i32>().is_ok() || // packet id
                !vec[2].parse::<f32>().is_ok() || // start voltage
                !vec[3].parse::<f32>().is_ok() || // end voltage
                !vec[4].parse::<f32>().is_ok() {  // resolution
                return Err("[parse_packet_string] Invalid packet parameter types.".into());
            }
            // build the CommandPacket
            let command_packet = CommandPacket::new(
                vec[1].parse::<i32>().unwrap(), 
                PacketCommand::TEST, 
                vec!(
                    vec[2].parse::<f32>().unwrap(),
                    vec[3].parse::<f32>().unwrap(),
                    vec[4].parse::<f32>().unwrap()
                )
            );
            // verify it
            match command_packet.verify_packet() {
                Ok(_) => Ok(command_packet),
                Err(err) => Err(err)
            }
        } else {
            Err("[parse_packet_string] Invalid packet type.".into())
        }
    }

    /// verify_packet makes sure the internals are valid.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * Nothing on success, an error on failure.
    pub fn verify_packet(&self) -> Result<()> {
        // check to see if packet id is a nonnegative integer
        if self.packet_id < 0 {
            return Err("[verify_packet] Packet ID must be a nonnegative integer.".into());
        }
        // check to see if packet params, if they exist, follow the following rules:
        // 1) there are exactly three parameters
        let length = self.packet_params.len();
        if (self.packet_command == PacketCommand::TEST) && (length != 3) {
            return Err("[verify_packet] Exactly three parameters are required for TEST.".into());
        } else if (self.packet_command != PacketCommand::TEST) && (length != 0) {
            return Err("[verify_packet] Exactly zero parameters are required for START or END.".into());
        }
        // for TEST commands
        if self.packet_command == PacketCommand::TEST {
            // 2) parameter 2 is strictly greater than parameter 1
            if self.packet_params[1] <= self.packet_params[0] {
                return Err("[verify_packet] Voltage End [1] should be strictly >= Voltage Start [0].".into());
            }
            // 3) parameter 3 is strictly less than or equal to the difference between parameter 2 and 1 and positive.
            if (self.packet_params[2] > (self.packet_params[1] - self.packet_params[0])) && (self.packet_params[2] > 0.0) {
                return Err("[verify_packet] Voltage Resolution [2] should be in the range (0, Voltage End - Voltage Start].".into());
            }
        }
        
        Ok(())
    }

    /// transmit_packet sends a command packet stringified over USB to the board.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// * `port` - port to send message to via serial
    /// 
    /// # Returns
    /// 
    /// * Nothing on success, an error on failure.
    pub fn transmit_packet(&self, port: &mut Port) -> Result<()> {
        // convert CommandPacket to string for transmission
        let mut message = String::from("");
        if self.packet_command == PacketCommand::TEST {
            message.push_str("TEST ");
        } else if self.packet_command == PacketCommand::START {
            message.push_str("START ");
        } else if self.packet_command == PacketCommand::END {
            message.push_str("END ");
        } else {
            return Err("[transmit_packet] invalid packet command enum.".into());
        }
        message.push_str(&self.packet_id.to_string());
        message.push_str(" ");
        
        for val in &self.packet_params {
            message.push_str(&val.to_string());
            message.push_str(" ");
        }
        message.push_str(";");

        // send message
        match send_message(port, message) {
            Ok(()) => Ok(()),
            Err(err) => Err(err.into())
        }
    }

    /// receive_packet grabs a string over USB from the board and converts it into a CommandPacket.
    /// Unused.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// * `port` - port to receive message from via serial
    /// 
    /// # Returns
    /// 
    /// * A CommandPacket on success, an error on failure.
    pub fn _receive_packet(&self, port: &mut Port) -> Result<CommandPacket> {
        // receive message
        match receive_message(port) {
            Ok(res) => {
                // Do string parsing and verification before return
                match CommandPacket::parse_packet_string(res) {
                    Ok(res) => Ok(res),
                    Err(err) => Err(err.into())
                }
            },
            Err(err) => Err(err.into())
        }
    }

    /// stringify converts the CommandPacket into a string.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * A String of the CommandPacket. Should be possible to regenerate using parse_packet_string().
    pub fn stringify(&self) -> String {
        format!("TEST {} {} {} {}\n", self.packet_id, self.packet_params[0], self.packet_params[1], self.packet_params[2])
    }
}



/// the DataPacket struct is generated when parsing UART or file data and represents a single data point.
/// it contains the following:
/// 
/// * packet id, representing the CommandPacket that ordered the execution of the regime this data packet belongs to
/// * packet subid, representing where along the testing regime that this packet belongs to
/// * packet type - what measurement type was performed and is in this data packet
/// * packet data - packet data associated with packet type.
/// 
/// We should expect a data packet for each sensor type in order of packet subid.
pub struct DataPacket {
    pub packet_id: i32,                 // corresponds to command_packet id that this data belongs to
    pub packet_subid: i32,              // corresponds to which measurement along the test regime the packet belongs to
    pub packet_type: PacketType,        // what measurement type was performed
    pub packet_data: f32                // data
}
impl DataPacket {
    pub fn new(packet_id: i32, packet_subid: i32, packet_type: PacketType, packet_data: f32) -> DataPacket {
        DataPacket {
            packet_id,
            packet_subid,
            packet_type: packet_type,
            packet_data
        }
    }

    /// parse_packet_string parses and verifies a string and converts it into a DataPacket if applicable.
    /// 
    /// # Arguments
    /// 
    /// * `string` - string to parse and verify
    /// 
    /// # Returns
    /// 
    /// * A DataPacket on success, an error on failure.
    pub fn parse_packet_string(string: String) -> Result<DataPacket> {
        let args = string.split(" ");
        let vec: Vec<&str> = args.collect();
        // data packet
        if vec[0] == "DATA" {
            // check for correct parameter types
            if  !vec[1].parse::<i32>().is_ok() ||   // packet id
                !vec[2].parse::<i32>().is_ok() ||   // subpacket id
                !vec[3].parse::<i32>().is_ok() ||   // measurement type
                !vec[4].parse::<f32>().is_ok() {    // measurement value
                return Err("Invalid packet parameter.".into())
            }
            // parse packet measurement type
            let measurement_type = vec[3].parse::<i32>().unwrap();
            let packet_type = PacketType::num_to_packet_type(measurement_type);
            // build the DataPacket
            let data_packet = DataPacket::new(
                vec[1].parse::<i32>().unwrap(), 
                vec[2].parse::<i32>().unwrap(), 
                packet_type, 
                vec[4].parse::<f32>().unwrap()
            );
            // verify it
            match data_packet.verify_packet() {
                Ok(_) => Ok(data_packet),
                Err(err) => Err(err)
            }
        } else {
            Err("Invalid packet type.".into())
        }
    }

    /// verify_packet makes sure the internals are valid.
    /// TODO: this function
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * Nothing on success, an error on failure.
    pub fn verify_packet(&self) -> Result<()> {
        // check to see if packet id is a nonnegative integer
        if self.packet_id < 0 {
            return Err("[verify_packet] Packet ID must be a nonnegative integer.".into());
        }
        // check to see if packet subid is a nonnegative integer
        if self.packet_subid < 0 {
            return Err("[verify_packet] Packet subID must be a nonnegative integer.".into());
        }

        Ok(())
    }

    /// transmit_packet sends a data packet stringified over USB to the board.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// * `port` - port to send message to via serial
    /// 
    /// # Returns
    /// 
    /// * Nothing on success, an error on failure.
    pub fn _transmit_packet(&self, port: &mut Port) -> Result<()> {
        // convert CommandPacket to string for transmission
        // TODO: this
        let message = String::from("HELLO");
        // send message
        match send_message(port, message) {
            Ok(()) => Ok(()),
            Err(err) => Err(err.into())
        }
    }

    /// receive_packet grabs a string over USB from the board and converts it into a DataPacket.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// * `port` - port to receive message from via serial
    /// 
    /// # Returns
    /// 
    /// * A DataPacket on success, an error on failure.
    pub fn _receive_packet(&self, port: &mut Port) -> Result<DataPacket> {
        // receive message
        match receive_message(port) {
            Ok(res) => {
                // Do string parsing and verification before return
                match DataPacket::parse_packet_string(res) {
                    Ok(res) => {
                        Ok(res)
                    },
                    Err(err) => Err(err.into())
                }
            },
            Err(err) => Err(err.into())
        }
    }

    /// stringify converts the DataPacket into a string.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * A String of the CommandPacket. Should be possible to regenerate using parse_packet_string().
    pub fn stringify(&self) -> String {
        format!("DATA {} {} {} {}\n", self.packet_id, self.packet_subid, self.packet_type.to_num(), self.packet_data)
    }
}



/// a PacketSet struct represents an agglomerate set of data for a single test regime.
/// It contains a single command packet and the data packets that were generated by the command packet from the Nucleo.
pub struct PacketSet {
    pub command_packet: CommandPacket,
    pub data_packets: Vec<DataPacket>,
}
impl PacketSet {
    /// save_packet_set saves the packet set as a file.
    /// 
    /// # Arguments
    /// 
    /// * `string` - string to parse and verify
    /// 
    /// # Returns
    /// 
    /// * A DataPacket on success, an error on failure.
    pub fn save_packet_set(&self) -> Result<()> {
        // generate file name
        let mut file_path: String = "test/".to_owned();
        let now: DateTime<Utc> = Utc::now();
        file_path.push_str(&format!("{}_", now));
        file_path.push_str(&self.command_packet.packet_id.to_string());
        file_path.push_str(".log");
        let f = File::create(file_path.clone())?;
        let mut f = BufWriter::new(f);

        // write header
        f.write_all(format!("{}\n", return_header()).as_bytes())?;
        // write command packet
        f.write_all(self.command_packet.stringify().as_bytes())?;
        // write start command
        f.write_all(format!("START {}\n", self.command_packet.packet_id).as_bytes())?;
        // write data packets
        let range = self.data_packets.len();
        for idx in 0..range {
            f.write_all(self.data_packets[idx].stringify().as_bytes())?;
        }
        // write end command
        f.write_all(format!("END {}\n", self.command_packet.packet_id).as_bytes())?;

        println!("[save_packet_set] Log file generated at {}.", file_path);

        Ok(())
    }

    /// visualize creates a PNG visualization of the IV curve with its data sets.
    /// 
    /// # Arguments
    /// 
    /// * `self`
    /// 
    /// # Returns
    /// 
    /// * Nothing
    pub fn visualize(&self) {
        // separate data into types
        let mut series_current = vec!();    // bounds: 0 - 7500 mA
        let mut series_power = vec!();      // bounds: 0 - 7500 mW
        let mut series_temp = vec!();       // bounds: 0 - 1100 G
        let mut series_irrad = vec!();      // bounds: 0 - 1100 C/10
                                            // voltage bounds is always 0 - 750 mV
        let mut subid:i32 = -1;
        let mut voltage:f32 = -1.0;

        let len = self.data_packets.len();
        for idx in 0..len {
            // if new packet subid
            if self.data_packets[idx].packet_subid != subid {
                subid = self.data_packets[idx].packet_subid;
                voltage = -1.0;
            }
            if self.data_packets[idx].packet_type == PacketType::VOLTAGE {
                voltage = self.data_packets[idx].packet_data;
            }
            if voltage != -1.0 {
                if self.data_packets[idx].packet_type == PacketType::CURRENT {
                    series_current.push((
                        (voltage*1000.0) as i32, 
                        (self.data_packets[idx].packet_data*1000.0) as i32
                    ));
                    series_power.push((
                        (voltage*1000.0) as i32,
                        (self.data_packets[idx].packet_data*1000.0*voltage) as i32
                    ));
                } else if self.data_packets[idx].packet_type == PacketType::TEMP {
                    series_temp.push((
                        (voltage*1000.0) as i32,
                        (self.data_packets[idx].packet_data*10.0) as i32
                    ));
                } else if self.data_packets[idx].packet_type == PacketType::IRRAD {
                    series_irrad.push((
                        (voltage*1000.0) as i32, 
                        self.data_packets[idx].packet_data as i32
                    ))
                }
            }
        }

        visualize(
            self.command_packet.packet_id, 
            self.command_packet.packet_params.clone(),
            series_current,
            series_power,
            series_temp,
            series_irrad
        );
    }
}
/// parse_buffer is a helper function for parse_file that attempts to extract a data or command packet from the string.
/// 
/// # Arguments
/// 
/// * `buffer` - A string potentially containing a data or command packet to be extracted.
/// 
/// # Returns
/// 
/// * A tuple of packet options on success, an error on failure.
fn parse_buffer(buffer: String) -> Result<(Option<CommandPacket>, Option<DataPacket>)> {
    if let Ok(data_packet) = DataPacket::parse_packet_string(buffer.clone()) {
        Ok((None, Some(data_packet)))
    } else if let Ok(command_packet) = CommandPacket::parse_packet_string(buffer) {
        Ok((Some(command_packet), None))
    } else {
        Err("Neither packet type was found.".into())
    }
}

/// parse_file takes a file path and attempts to parse a coherent* set of packets from the file data.
/// *coherent - packets are in a distinct order, are of the right format, and with a correct header.
/// 
/// # Arguments
/// 
/// * `file_path` - A string representing the file to open and parse.
/// 
/// # Returns
/// 
/// * A vector of packets on success, or an error on failure.
pub fn parse_file(file_path: String) -> Result<Vec<PacketSet>> {
    // check if valid (exists, has correct header, etc)
    if !Path::new(&file_path).is_file() {
        return Err("Is not a file. Retry.".into());
    }

    let mut f = BufReader::new(File::open(&file_path).unwrap());
    let mut buffer = String::new(); 
    // open and read the first line looking for a valid header
    f.read_line(&mut buffer).unwrap();
    if buffer.trim() != return_header() {
        return Err("Invalid header {}".into());
    }
    println!("[parse_file] Matched the header.");

    let mut packet_sets:Vec<PacketSet> = vec!();
    let mut end = false;
    buffer = String::new();
    while !end {
        // TODO: set a sigint handler for gracefully exiting
        // read a line, if any
        let size = match f.read_line(&mut buffer) {
            Ok(size) => size,
            Err(_) => {
                return Err("[parse_file] Read error".into());
            }
        };
        // parse and match the line
        if size > 0 {
            match parse_buffer(buffer.trim().to_string()) {
                Ok(res) => {
                    // assume if one works the other won't
                    if let Some(command_packet) = res.0 {
                        // check to see if a TEST command packet already exists for the same ID
                        let mut found = false;
                        for packet in &packet_sets {
                            if (packet.command_packet.packet_id == command_packet.packet_id) && (command_packet.packet_command == PacketCommand::TEST){
                                found = true;
                            }
                        }
                        // if it doesn't exist, make a new PacketSet
                        if !found && command_packet.packet_command == PacketCommand::TEST {
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
                // we'll ignore invalid packets during parsing.
                Err(err) => println!("{}", err)
            }
            buffer = String::new();
        } else {
            println!("[parse_file] EOF.");
            end = true;
        }
    }
    println!("[parse_file] Packets successfully parsed.");
    Ok(packet_sets)
}

/// return_header is used by parse_file to check for a correct header. Log files need to match this string for correct parsing.
/// 
/// # Returns
/// 
/// * The header string.
fn return_header() -> String {
    String::from("Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.")
}