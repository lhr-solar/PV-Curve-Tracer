use plotters::prelude::*;
use crate::{PacketSet, PacketType, PacketCommand};

const IMAGE_WIDTH:u32 = 900;
const IMAGE_HEIGHT:u32 = 600;

pub fn visualize_packets(packet_sets: Vec<PacketSet>) {
    // generate an image for each regime test
    for packet_set in packet_sets {
        if packet_set.command_packet.packet_command == PacketCommand::TEST {
            // separate data into types
            let mut series_current = vec!();    // bounds: 0 - 7500 mA
            let mut series_power = vec!();      // bounds: 0 - 7500 mW
            let mut series_temp = vec!();       // bounds: 0 - 1100 G
            let mut series_irrad = vec!();      // bounds: 0 - 1100 C/10
                                                // voltage bounds is always 0 - 750 mV
            let mut subid:i32 = -1;
            let mut voltage:f32 = -1.0;
            for packet in &packet_set.data_packets {
                // if new packet subid
                if packet.packet_subid != subid {
                    subid = packet.packet_subid;
                    voltage = -1.0;
                }
                if packet.packet_type == PacketType::VOLTAGE {
                    voltage = packet.packet_data;
                }
                if voltage != -1.0 {
                    if packet.packet_type == PacketType::CURRENT {
                        series_current.push((
                            (voltage*1000.0) as i32, 
                            (packet.packet_data*1000.0) as i32
                        ));
                        series_power.push((
                            (voltage*1000.0) as i32,
                            (packet.packet_data*1000.0*voltage) as i32
                        ));
                    } else if packet.packet_type == PacketType::TEMP {
                        series_temp.push((
                            (voltage*1000.0) as i32,
                            (packet.packet_data*10.0) as i32
                        ));
                    } else if packet.packet_type == PacketType::IRRAD {
                        series_irrad.push((
                            (voltage*1000.0) as i32, 
                            packet.packet_data as i32
                        ))
                    }
                }
            }

            // visualize with plotters
            // generate file name
            let mut file_path: String = "img/".to_owned();
            file_path.push_str(&packet_set.command_packet.packet_id.to_string());
            file_path.push_str(".png");
            // create the canvas
            let root_drawing_area = BitMapBackend::new(&file_path, (IMAGE_WIDTH, IMAGE_HEIGHT)).into_drawing_area();
            // set canvas as white
            root_drawing_area.fill(&WHITE).unwrap();
            
            // generate image name
            let mut image_name: String = "Test Regime for [".to_owned();
            image_name.push_str(&packet_set.command_packet.packet_params[0].to_string());
            image_name.push_str(", ");
            image_name.push_str(&packet_set.command_packet.packet_params[1].to_string());
            image_name.push_str(", ");
            image_name.push_str(&packet_set.command_packet.packet_params[2].to_string());
            image_name.push_str("]");
            let root_drawing_area = root_drawing_area.titled(&image_name, ("sans-serif", 30).into_font()).unwrap();
            let (left, right) = root_drawing_area.split_horizontally(IMAGE_WIDTH/2);

            // build left graph context
            let mut ctx = ChartBuilder::on(&left)
                .caption("Current and Power as a Function of Voltage", ("Arial", 20))
                .x_label_area_size(40)
                .y_label_area_size(50)
                .margin(5)
                .build_ranged(0..750, 0..7500)
                .unwrap();

            ctx.configure_mesh()
                .y_desc("Current (mA), Power (mW)")
                .x_desc("Voltage (mV)")
                .axis_desc_style(("Arial", 13))
                .draw().unwrap();
            
            // plot left graph data
            // current
            ctx.draw_series(
                series_current.iter().map(|point| TriangleMarker::new(*point, 4, &BLUE)),
            ).unwrap();
            // power
            ctx.draw_series(
                series_power.iter().map(|point| Circle::new(*point, 4, &RED)),
            ).unwrap();

            // build right graph context
            let mut ctx2 = ChartBuilder::on(&right)
                .caption("Irrad and Temp as a Function of Voltage", ("Arial", 20))
                .x_label_area_size(40)
                .y_label_area_size(50)
                .margin(5)
                .build_ranged(0..750, 0..1100)
                .unwrap();

            ctx2.configure_mesh()
                .y_desc("Irradiance (G), Temp (C*10)")
                .x_desc("Voltage (mV)")
                .axis_desc_style(("Arial", 13))
                .draw().unwrap();
            
            // plot right graph data
            // irradiance
            ctx2.draw_series(
                series_irrad.iter().map(|point| TriangleMarker::new(*point, 4, &BLACK)),
            ).unwrap();
            // temperature
            ctx2.draw_series(
                series_temp.iter().map(|point| Circle::new(*point, 4, &GREEN)),
            ).unwrap();

            println!("Image generated.");
        }
    }


}