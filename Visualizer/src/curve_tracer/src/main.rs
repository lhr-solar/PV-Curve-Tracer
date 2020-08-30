use plotters::prelude::*;
use terminal_menu::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /**
        To start with, we want to do the following things:
        1. Ask the user what he/she wants to do:
            a. do you want to visualize an existing file? (go to 2a)
            b. or do you want to send a command to the PV Curve Tracer Board? (go to 3a)
        2a. Ask for the file to parse
        2b. Process and open the output image. Alternatively there could be some realtime
            update setup.
        3a. Setup a menu (GUI/CLI) of commands to send.
        3b. After command has been selected, give further instructions that may be required
            (i.e. tell the user to adjust the rotary switch and connect the array)
        3c. Prompt for permission to start execution
        3d. Transmit command
        3e. Busywait for messages to come in, reorganize, and parse them.
        3f. See 2b.
        3g. Jump back to 1 or have some further functionality...
    */

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
    
    //if 1a is chosen
    if selection_value(&menu_main, "Selection") == "Visualize Data from Preexisting File" {
        // prompt for file to parse
        let mut file_path = String::new();
        println!("Enter a valid file to visualize: ");
        std::io::stdin().read_line(&mut file_path).unwrap();
        println!("Filepath: {}", file_path);
        // TODO: check if valid (exists, has correct header, etc)
        // TODO: if not valid either return to the main menu or reprompt for file or exit
        // TODO: else if valid load in the file and visualize it with plotters
    }
    
    //else 1b is chosen
    else if selection_value(&menu_main, "Selection") == "Send Command to Curve Tracer and Collect Data" {
        // TODO: create a new menu for selecting command and command data
        let menu_command = menu(vec![
            label("(use arrow keys or wasd)"),
            list("TEST TYPE", vec!["CELL", "MODULE", "ARRAY"]),
            numeric("Starting Voltage", 1.0, Some(1.0), Some(0.0), Some(10.0)),
            numeric("Ending Voltage", 9.0, Some(1.0), Some(0.0), Some(10.0)),
            numeric("Resolution", 1.0, Some(1.0), Some(0.0), Some(10.0)),
            button("Done")
        ]);
        //open the menu
        activate(&menu_command);
        //wait for the menu to exit
        wait_for_exit(&menu_command);
        //read values
        println!("Selection:     {}", selection_value(&menu_command, "TEST TYPE"));
        // TODO: when complete ask for confirmation
        // TODO: warn about operating procedures
        // TODO: ask for final okay
        // TODO: execute and wait for the packets to roll in
        // TODO: in the meantime display or wait until last packet to display
        // TODO: give option to save
        // TODO: return to main menu or ask to execute another command
    }
    // else exit

    Ok(())
}