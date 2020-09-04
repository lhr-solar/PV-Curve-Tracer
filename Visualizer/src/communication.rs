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
    // open the port
    match open_serial_comm() {
        Ok(mut port) => {
            // test
            // if let Ok(_) = send_message(&mut port, String::from("Hello world.")) {
                let mut iter = 0;
                let mut string_idx = 0;
                let strings = [
                    "Hello",
                    " world",
                    ".",
                    "Nice to meet you."
                ];
                loop {
                    // testing for output
                    if let Ok(msg) = receive_message(&mut port) {
                        print!("{}", msg);
                    }

                    if iter == 0 {
                        if string_idx < 4 {
                            if let Ok(_) = send_message(&mut port, String::from(strings[string_idx])) {}
                            string_idx += 1;
                        }
                    }
                    iter = (iter + 1)%5000;
                }
            // }
            return Ok(PacketSet {
                command_packet: command_packet,
                data_packets: vec!()
            })
            // // verify that it's correct
            // match command_packet.verify_packet() {
            //     Ok(()) => {
            //         // send the command
            //         match command_packet.transmit_packet(&mut port) {
            //             Ok(()) => {
            //                 // retrieve responses
            //                 let mut packet_set = PacketSet {
            //                     command_packet: command_packet,
            //                     data_packets: vec!()
            //                 };
            //                 // TODO: add some fancy progress bar here and loop until last expected packet is found

            //                 // SAMPLE CODE
            //                 for iteration in 1..5 {
            //                     println!("Iteration: {}", iteration.to_string());
            //                     // create generic packet
            //                     let data_packet = DataPacket::new(-1, -1, PacketType::VOLTAGE, -1.0);
            //                     // retrieve data
            //                     if let Ok(_) = data_packet.receive_packet(&mut port) {
            //                         // TODO: look for ending packet or parse for it
            //                         if let Ok(_) = data_packet.verify_packet() {
            //                             // TODO: do the same manip as in parse_file
            //                             packet_set.data_packets.push(data_packet);
            //                         }
            //                     }
            //                     thread::sleep(Duration::from_millis(5000));
            //                 }
            //                 // return the completed packets
            //                 Ok(packet_set)
            //             },
            //             Err(err) => Err(format!("{}", err).into())
            //         }
            //     },
                // Err(err) => Err(format!("{}", err).into())
            // }
        },
        Err(err) => {
            Err(format!("{}", err).into())
        }
    }
}

