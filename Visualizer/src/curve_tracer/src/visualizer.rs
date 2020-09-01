use plotters::prelude::*;
use crate::{PacketSet, PacketType};

pub fn visualize_packets(packet_set: Vec<PacketSet>) {
    // visualize with plotters
    // generate file name
    let mut file_path: String = "img/".to_owned();
    file_path.push_str(&packet_set[1].command_packet.packet_id.to_string());
    file_path.push_str(".png");
    // create the canvas
    let root_drawing_area = BitMapBackend::new(&file_path, (600, 400)).into_drawing_area();
    // set canvas as white
    root_drawing_area.fill(&WHITE).unwrap();
    // build context
    let mut ctx = ChartBuilder::on(&root_drawing_area)
        .caption("Current as a function of voltage", ("Arial", 20))
        .x_label_area_size(40)
        .y_label_area_size(50)
        .margin(5)
        .build_ranged(0..800, 0..8000)
        .unwrap();

    ctx.configure_mesh()
        .y_desc("Current (mA)")
        .x_desc("Voltage (mV)")
        .axis_desc_style(("Arial", 10))
        .draw().unwrap();

    // separate data into types
    let mut series_current = vec!();
    let mut subid:i32 = -1;
    let mut voltage:f32 = -1.0;
    for packet in &packet_set[1].data_packets {
        // if new packet subid
        if packet.packet_subid != subid {
            subid = packet.packet_subid;
            voltage = -1.0;
        }
        if packet.packet_type == PacketType::VOLTAGE {
            voltage = packet.packet_data;
        }
        if packet.packet_type == PacketType::CURRENT {
            if voltage != -1.0 {
                series_current.push((
                    (voltage*1000.0) as i32, 
                    (packet.packet_data*1000.0) as i32
                ));
            }
        }
    }
    
    // plot data
    ctx.draw_series(
        series_current.iter().map(|point| TriangleMarker::new(*point, 5, &BLUE)),
    ).unwrap();

    println!("Image generated.");
}