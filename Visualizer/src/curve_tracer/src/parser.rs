/// File: parser.rs
/// Author: Matthew Yu
/// Organization: UT Solar Vehicles Team
/// Date Created: 8/29/20
/// Last Modified: 8/31/20
/// Description: This file parses either a file or packet data send via UART and fills a struct.
///     This struct is used later for visualization or storage.

pub enum PacketCommand {
    START,
    TEST,
}

pub enum PacketType {
    VOLTAGE,
    CURRENT,
    TEMP,
    IRRAD,
}

pub struct CommandPacket {
    packet_id: i32,                 // identifier for the packet
    packet_command: PacketCommand,  // contains command type [START/TEST]
    packet_params: Vec<i64>         // contains optional command data [voltage start, voltage end, resolution]
}

pub struct DataPacket {
    packet_id: i32,                 // corresponds to command_packet id that this data belongs to
    packet_subid: i32,              // corresponds to which measurement along the test regime the packet belongs to
    packet_type: PacketType,        // what measurement type was performed
    packet_data: f32                // data
}

pub struct PacketSet {
    command_packet: CommandPacket,
    data_packets: Vec<DataPacket>,
}

impl CommandPacket {
    /// verify_packet makes sure the internals are valid.
    /// returns true if correct.
    pub fn verify_packet(&self) -> bool {
        // TODO: this
        false
    }

    /// transmit_packet sends a command packet over USB to the board.
    /// returns true if successfully sent.
    pub fn transmit_packet(&self) -> bool {
        // TODO: this
        false
    }

    /// receive_packet is an unused method.
    pub fn receive_packet(&self) -> bool {
        false
    }
}

impl DataPacket {
    /// verify_packet makes sure the internals are valid.
    /// returns true if correct.
    pub fn verify_packet(&self) -> bool {
        // TODO: this
        false
    }
}