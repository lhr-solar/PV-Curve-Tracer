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

use pbr::ProgressBar;
use std::{
    error,
    thread,
    time::Duration,
};

use crate::{
    parser::*,
    port::*
};

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// execute_test attempts to start a test regime on the Nucleo and grab the returned data.
/// 
/// # Arguments
/// 
/// * `command_packet` - CommandPacket with the command to send to the Nucleo
/// 
/// # Returns
/// 
/// * A string on success, an error on failure.
pub fn execute_test(command_packet: CommandPacket) -> Result<PacketSet> {
    // A couple of things should be done in order to perform and collect data from a test regime:
    // 1) We need to open the serial communications port
    // 2) The program sends the test regime command (i.e. CMD [ID] [START_VOLTAGE] [END_VOLTAGE] [VOLTAGE_RESOLUTION])
    // 3) The program checks if the user is ready, and then sends the START [ID] command. The nucleo begins processing the test regime associated with that ID.
    // 4) The nucleo begins sending back data in the format DATA [ID] [SUBID] [MEASUREMENT_TYPE] [MEASUREMENT_DATA].
    // 5) The nucleo completed data transfer by submitting the end command. END [ID].

    // 0) preprocessing: verify that the command packet is correct
    if let Err(err) = command_packet.verify_packet() {
        return Err(err);
    }
    let cmd_id = command_packet.packet_id.clone();
    let cmd_args = command_packet.packet_params.clone();

    // 1) open the port
    let port = open_serial_comm();
    if let Err(err) = port {
        return Err(err);
    }

    // 2) send the command
    let mut port = port.unwrap(); // okay since we handled the err case earlier
    if let Err(err) = command_packet.transmit_packet(&mut port) {
        return Err(err);
    }
    println!("\nCommand packet sent to the PV Curve Tracer Board.");

    // 3) check to see if the user is ready
    println!("Are you ready to begin execution? (Y/abort) ");
    let mut response = String::from("");
    std::io::stdin().read_line(&mut response).unwrap();
    if response != "Y\n" {
        return Err("[execute_test] Aborting execution.".into());
    } else {
        println!("[execute_test] Beginning execution.");
    }
    // and send the start command
    if let Err(err) = CommandPacket::new(cmd_id.clone(), PacketCommand::START, vec!()).transmit_packet(&mut port) {
        return Err(err);
    }

    // 4) begin retrieving data
    let mut packet_set = PacketSet {
        command_packet: command_packet,
        data_packets: vec!()
    };

    // initiate progress bar - estimate the number of subID groups that'll need to be collected based on the command
    let count = ((cmd_args[1] - cmd_args[0])/cmd_args[2]) as u64 + 1;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");

    // while we haven't received the end command
    let mut end = true; // set to true for testing
    while !end {
        // TODO: option for user sigint

        // retrieve packet, if any
        match receive_message(&mut port) {
            Ok(res) => {
                let res_copy = res.clone();
                let res_vec:Vec<&str> = res_copy.split(' ').collect();

                // if res is a DataPacket, add to the packet_set
                if let Ok(data_packet) = DataPacket::parse_packet_string(res) {
                    // TODO: check for subid and update the progress bar
                    pb.inc();
                    
                    packet_set.data_packets.push(data_packet);
                }
                // if res is an END command with a matching id, set end to true
                else if (res_vec[0] == "END") && (res_vec[1].parse::<i32>().unwrap() == cmd_id) {
                    end = true;
                }
                // else print invalid packet type error
                else {
                    println!("[execute_test] Invalid packet type.");
                }
            },
            Err(err) => println!("[execute_test] {}", err)
        }
    }

    // TODO: test
    for _ in 0..count {
        pb.inc();
        thread::sleep(Duration::from_millis(20));
    }
    // complete the progress bar
    pb.finish_print("All packets received.");

    Ok(packet_set)
}

