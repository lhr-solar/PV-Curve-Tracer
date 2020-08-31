/// File: main.rs
/// Author: Matthew Yu
/// Organization: UT Solar Vehicles Team
/// Date Created: 8/29/20
/// Last Modified: 8/31/20
/// Description: This file runs the main CLI application managing the PV Curve Tracer.
///     This program is able to send commands to the STM32 Nucleo over USB, recieve data
///     packets, and visualize PV curves from those packets or from a log file.

mod visualizer;
use visualizer::*;
mod parser;
use parser::*;
use terminal_menu::*;
use std::{
    error,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    sync::{Arc, RwLock},
};


type TerminalMenu = Arc<RwLock<TerminalMenuStruct>>;
// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    // To start with, we want to do the following things:
    // 1. Ask the user what he/she wants to do:
    //     a. do you want to visualize an existing file? (go to 2a)
    //     b. or do you want to send a command to the PV Curve Tracer Board? (go to 3a)
    //     c. exit
    // 2a. Ask for the file to parse
    // 2b. Process and open the output image. Alternatively there could be some realtime
    //     update setup.
    // 3a. Setup a menu (GUI/CLI) of commands to send.
    // 3b. After command has been selected, give further instructions that may be required
    //     (i.e. tell the user to adjust the rotary switch and connect the array)
    // 3c. Prompt for permission to start execution
    // 3d. Transmit command
    // 3e. Busywait for messages to come in, reorganize, and parse them.
    // 3f. See 2b.
    // 3g. Jump back to 1

    // program header text
    println!(
        "PV Curve Tracer Visualizer and Command Center 0.1.0.\n
        Developed by Matthew Yu (2020).\n");

    let mut menu_result = main_menu();
    let mut result = selection_value(&menu_result, "Selection");
    while result != "Exit" {
        // if 1a is chosen
        if selection_value(&menu_result, "Selection") == "Visualize Data from Preexisting File" {   
            file_selection_menu();
        }
        // else 1b is chosen
        else if selection_value(&menu_result, "Selection") == "Send Command to Curve Tracer and Collect Data" {
            let menu_result = command_menu();
            let selection_result = selection_value(&menu_result, "TEST TYPE");
            let submenu_result:TerminalMenu;
            // set variables
            if selection_result == "CELL" {
                submenu_result = get_submenu(&menu_result, "CELL Test Parameters");
            } else if selection_result == "MODULE" {
                submenu_result = get_submenu(&menu_result, "MODULE Test Parameters");
            } else { // ARRAY
                submenu_result = get_submenu(&menu_result, "ARRAY Test Parameters");
            }
            let voltage_start = numeric_value(&submenu_result, "Starting Voltage (mV)");
            let voltage_end = numeric_value(&submenu_result, "Ending Voltage (mV)");
            let voltage_resolution = numeric_value(&submenu_result, "Resolution (mV)");

            // error check bounds
            if voltage_start >= voltage_end {
                println!("Out of bounds error regarding voltage start and end params.");
                println!("Aborting.");
            } else {
                // when complete display results and ask for confirmation
                println!("Selection:\t{}", selection_result);
                println!("Selected Parameters:");
                println!("Start Voltage:\t\t{}", voltage_start);
                println!("End Voltage:\t\t{}", voltage_end);
                println!("Voltage Resolution:\t{}", voltage_resolution);
                println!("Are these parameters correct? (Y/n)");

                let mut response = String::from("");
                std::io::stdin().read_line(&mut response).unwrap();
                if response == "Y\n" {
                    // warn about operating procedures
                    print_disclaimer();
                    println!("Please rotate the rotary switch to {} mode labeled on the board.", selection_result);
                    println!("Now, connect the PV to the terminals of the board.");
                    // ask for final okay
                    response = String::from("");
                    println!("Are you ready to begin execution? (Y/abort) ");
                    std::io::stdin().read_line(&mut response).unwrap();
                    if response == "Y\n" {
                        println!("Starting execution.");
                        // TODO: execute and wait for the packets to roll in
                        // TODO: in the meantime display or wait until last packet to display
                        // TODO: give option to save
                    } else {
                        println!("Aborting.");
                    }
                } else {
                    println!("Aborting.");
                }
            }
        }

        menu_result = main_menu();
        result = selection_value(&menu_result, "Selection");
    }
    // Exit
    Ok(())
}

fn main_menu() -> TerminalMenu {
    //create the menu for 1a/b
    let menu_main = menu(vec![
        label("(use arrow keys or wasd)"),
        scroll("Selection", vec![
            "Visualize Data from Preexisting File", 
            "Send Command to Curve Tracer and Collect Data",
            "Exit"]),
        button("Done Selecting.")
    ]);
    //open the menu
    activate(&menu_main);
    //wait for the menu to exit
    wait_for_exit(&menu_main);

    menu_main
}

fn file_selection_menu() {
    // prompt for file to parse
    let mut file_path = String::from("");
    while file_path != "exit" {
        // reset file_path variable
        file_path = String::from("");
        println!("Enter a valid file to visualize or type 'exit': ");
        std::io::stdin().read_line(&mut file_path).unwrap();
        // strip newline
        file_path = file_path[0..file_path.len()-1].to_string();
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
                    let packets:Vec<PacketSet> = vec!();
                    let mut success = false;
                    // TODO: then read in the rest, building a set of packet objects
                    while let Ok(result) = f.read_line(&mut buffer) {
                        if result != 0 {
                            println!("{}", buffer);
                            // TODO: load in the valid file and visualize it with plotters
                            // build valid packet
                            buffer = String::new();
                        } else {
                            println!("EOF.");
                            success = true;
                            break;
                        }
                    }

                    // successful parsing, gather up the packets and visualize it
                    if success {
                        // TODO: visualize packets
                        visualize_packets(packets);
                    }
                } else {
                    println!("Invalid header {}", buffer.trim());
                }
            } else {
                println!("Is not a file. Retry.");
            }
            println!("Filepath:\t{}", file_path);
        } else {
            println!("Exiting the file selection menu.");
        }
    }
}

fn command_menu() -> TerminalMenu {
    // create a new menu for selecting command and command data
    let menu_command = menu(vec![
        label("(use arrow keys or wasd. You only need to adjust the selected test type's parameters.)"),
        list("TEST TYPE", vec!["CELL", "MODULE", "ARRAY"]),
        submenu("CELL Test Parameters", {
            // range for a cell test is from 0 to .6 V
            vec![
                label("Adjust the test parameters (default [0:600mV:1mV]):"),
                numeric("Starting Voltage (mV)", 0.0, Some(1.0), Some(0.0), Some(600.0)),
                numeric("Ending Voltage (mV)", 600.0, Some(1.0), Some(0.0), Some(600.0)),
                numeric("Resolution (mV)", 1.0, Some(1.0), Some(1.0), Some(100.0)),
                back_button("Back")
            ]
        }),
        submenu("MODULE Test Parameters", {
            // range for a cell test is from 0 to 6 V
            vec![
                label("Adjust the test parameters (default [0:6000mV:1mV]):"),
                numeric("Starting Voltage (mV)", 0.0, Some(1.0), Some(0.0), Some(6000.0)),
                numeric("Ending Voltage (mV)", 6000.0, Some(1.0), Some(0.0), Some(6000.0)),
                numeric("Resolution (mV)", 1.0, Some(1.0), Some(1.0), Some(1000.0)),
                back_button("Back")
            ]
        }),
        submenu("ARRAY Test Parameters", {
            // range for a cell test is from 0 to 100 V
            vec![
                label("Adjust the test parameters (default [0:10,0000mV:1mV]):"),
                numeric("Starting Voltage (mV)", 0.0, Some(1.0), Some(0.0), Some(100000.0)),
                numeric("Ending Voltage (mV)", 100000.0, Some(1.0), Some(0.0), Some(100000.0)),
                numeric("Resolution (mV)", 1.0, Some(1.0), Some(1.0), Some(10000.0)),
                back_button("Back")
            ]
        }),
        
        button("Done")
    ]);
    //open the menu
    activate(&menu_command);
    //wait for the menu to exit
    wait_for_exit(&menu_command);

    menu_command
}

fn print_disclaimer() {
    println!("--------------------------------------------------");
    println!("|         IMPORTANT OPERATING PROCEDURES         |");
    println!("| Use with caution. UTSVT is not liable for any  |");
    println!("| damages or persons harmed during the execution |");  
    println!("| of the PV Curve Tracer.                        |");
    println!("|                                                |");
    println!("| 1) Rotate the rotary switch to the correct     |");
    println!("| operational mode prior to plugging in the PV.  |");
    println!("|                                                |");
    println!("| 2) Make sure you plug in the cell/module/array |");
    println!("| using electrically insulated gloves or when    |");
    println!("| the PV is under shade to prevent accidental    |");
    println!("| electrocution.                                 |");
    println!("|                                                |");
    println!("| 3) Connect the positive and negative ends of   |");
    println!("| the PV to the correctly labeled terminals of   |");
    println!("| the PV Tracer Board.                           |");
    println!("|                                                |");
    println!("| 4) While in operation, do NOT touch the conta- |");
    println!("| acts of the PV. Wait until the program finish- |");
    println!("| es execution, then shade the PV before discon- |");
    println!("| necting it.                                    |");
    println!("|                                                |");
    println!("| 5) Do NOT adjust the rotary switch during exe- |");
    println!("| cution of the program, or while the PV is con- |");
    println!("| nected. This will fry the voltage sensor.      |");
    println!("| ---------------------------------------------- |");
}

fn return_header() -> String {
    String::from("Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.")
}