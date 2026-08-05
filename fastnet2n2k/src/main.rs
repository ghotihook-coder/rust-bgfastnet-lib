use bgfastnet_lib::{decode_frame, DecodedFrame, project};
use log::{error, info, warn};
use serialport::{DataBits, Parity, StopBits};
use socketcan::{CanSocket, Socket};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process;
use std::time::Duration;

mod pgn_map;
use pgn_map::get_pgn_for_path;

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim().replace([' ', '\n', '\t', '\r'], "");
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn open_input(input: &str) -> Result<Box<dyn Read>, String> {
    if Path::new(input).exists() {
        let file = File::open(input)
            .map_err(|e| format!("Failed to open input file: {e}"))?;
        Ok(Box::new(BufReader::new(file)))
    } else {
        let serial = serialport::new(input, 28800)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::Two)
            .parity(Parity::Odd)
            .timeout(Duration::from_secs(1))
            .open()
            .map_err(|e| format!("Failed to open serial port: {e}"))?;
        Ok(Box::new(serial))
    }
}

fn open_output(output: &str) -> Result<Box<dyn Write>, String> {
    if output == "stdout" {
        Ok(Box::new(std::io::stdout()))
    } else if output == "can0" {
        let can = CanSocket::open("can0")
            .map_err(|e| format!("Failed to open CAN device: {e}"))?;
        Ok(Box::new(can))
    } else {
        let file = File::create(output)
            .map_err(|e| format!("Failed to create output file: {e}"))?;
        Ok(Box::new(file))
    }
}

fn format_n2k_output(pgn: u32, src: u8, data: &[u8]) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let data_hex: Vec<String> = data.iter().map(|b| format!("{:02X}", b)).collect();
    format!("{},3,{},{},{}",
            timestamp, pgn, src, data_hex.join(","))
}

fn decode_frame_to_n2k(frame: &DecodedFrame) -> Option<(u32, u8, Vec<u8>)> {
    let signal_k = project(frame);
    
    // Try to map each Signal K path to a PGN
    // PGN 127240 - Attitude (roll, pitch)
    if signal_k.contains_key("navigation.attitude.roll") || signal_k.contains_key("navigation.attitude.pitch") {
        let mut payload = [0u8; 8];
        let roll = *signal_k.get("navigation.attitude.roll").unwrap_or(&0.0);
        let pitch = *signal_k.get("navigation.attitude.pitch").unwrap_or(&0.0);
        payload[0..4].copy_from_slice(&(roll as f32).to_le_bytes());
        payload[4..8].copy_from_slice(&(pitch as f32).to_le_bytes());
        return Some((127240, 0, payload.to_vec()));
    }
    
    // PGN 127245 - Rate of Turn
    if signal_k.contains_key("navigation.rateOfTurn") {
        let mut payload = [0u8; 8];
        let rot = *signal_k.get("navigation.rateOfTurn").unwrap();
        payload[0..4].copy_from_slice(&(rot as f32).to_le_bytes());
        return Some((127245, 0, payload.to_vec()));
    }
    
    // PGN 127249 - Leeway
    if signal_k.contains_key("navigation.leewayAngle") {
        let mut payload = [0u8; 8];
        let leeway = *signal_k.get("navigation.leewayAngle").unwrap();
        payload[0..4].copy_from_slice(&(leeway as f32).to_le_bytes());
        return Some((127249, 0, payload.to_vec()));
    }
    
    // PGN 127250 - Heading
    if signal_k.contains_key("navigation.headingTrue") || signal_k.contains_key("navigation.headingMagnetic") {
        let mut payload = [0u8; 8];
        let heading = *signal_k.get("navigation.headingTrue").or(signal_k.get("navigation.headingMagnetic")).unwrap_or(&0.0);
        payload[0..4].copy_from_slice(&(heading as f32).to_le_bytes());
        payload[4] = 0xFF;
        return Some((127250, 0, payload.to_vec()));
    }
    
    // PGN 127251 - Vessel Heading
    if signal_k.contains_key("navigation.headingTrue") || signal_k.contains_key("navigation.headingMagnetic") || signal_k.contains_key("steering.rudderAngle") {
        let mut payload = [0u8; 8];
        let heading = *signal_k.get("navigation.headingTrue").or(signal_k.get("navigation.headingMagnetic")).unwrap_or(&0.0);
        payload[0..4].copy_from_slice(&(heading as f32).to_le_bytes());
        payload[4] = 0xFF;
        if let Some(&rudder) = signal_k.get("steering.rudderAngle") {
            payload[4..8].copy_from_slice(&(rudder as f32).to_le_bytes());
        }
        return Some((127251, 0, payload.to_vec()));
    }
    
    // PGN 127257 - Speed (Boat Speed)
    if signal_k.contains_key("navigation.speedThroughWater") {
        let mut payload = [0u8; 8];
        let speed = *signal_k.get("navigation.speedThroughWater").unwrap();
        payload[0..4].copy_from_slice(&(speed as f32).to_le_bytes());
        return Some((127257, 0, payload.to_vec()));
    }
    
    // PGN 128258 - Apparent Wind Speed
    if signal_k.contains_key("environment.wind.speedApparent") {
        let mut payload = [0u8; 8];
        let speed = *signal_k.get("environment.wind.speedApparent").unwrap();
        payload[0..4].copy_from_slice(&(speed as f32).to_le_bytes());
        return Some((128258, 0, payload.to_vec()));
    }
    
    // PGN 128259 - True Wind Speed
    if signal_k.contains_key("environment.wind.speedTrue") {
        let mut payload = [0u8; 8];
        let speed = *signal_k.get("environment.wind.speedTrue").unwrap();
        payload[0..4].copy_from_slice(&(speed as f32).to_le_bytes());
        return Some((128259, 0, payload.to_vec()));
    }
    
    // PGN 128267 - Water Depth
    if signal_k.contains_key("environment.depth.belowTransducer") {
        let mut payload = [0u8; 8];
        let depth = *signal_k.get("environment.depth.belowTransducer").unwrap();
        payload[0..4].copy_from_slice(&(depth as f32).to_le_bytes());
        return Some((128267, 0, payload.to_vec()));
    }
    
    // PGN 128275 - Distance
    if signal_k.contains_key("navigation.log") {
        let mut payload = [0u8; 8];
        let dist = *signal_k.get("navigation.log").unwrap();
        payload[0..4].copy_from_slice(&(dist as f32).to_le_bytes());
        return Some((128275, 0, payload.to_vec()));
    }
    
    // PGN 129025 - Position (Latitude)
    if signal_k.contains_key("navigation.position") {
        let mut payload = [0u8; 8];
        let pos = *signal_k.get("navigation.position").unwrap();
        payload[0..8].copy_from_slice(&pos.to_le_bytes());
        return Some((129025, 0, payload.to_vec()));
    }
    
    // PGN 129291 - COG & SOG
    if signal_k.contains_key("navigation.speedOverGround") && signal_k.contains_key("navigation.courseOverGroundTrue") {
        let mut payload = [0u8; 8];
        let sog = *signal_k.get("navigation.speedOverGround").unwrap();
        let cog = *signal_k.get("navigation.courseOverGroundTrue").unwrap();
        payload[0..4].copy_from_slice(&(sog as f32).to_le_bytes());
        payload[4..8].copy_from_slice(&(cog as f32).to_le_bytes());
        return Some((129291, 0, payload.to_vec()));
    }
    
    None
}

fn process_file_input<P: AsRef<Path>>(input_path: P, output: &mut Box<dyn Write>) {
    let file = File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let mut bytes = hex_to_bytes(&line);
        
        while bytes.len() >= 6 {
            let body_size = bytes[2] as usize;
            let frame_len = 5 + body_size + 1;
            
            if bytes.len() < frame_len {
                break;
            }
            
            let frame = &bytes[0..frame_len];
            match decode_frame(frame) {
                Ok(decoded) => {
                    info!("Decoded: {} -> {:?}", decoded.command, decoded.values);
                    if let Some((pgn, src, data)) = decode_frame_to_n2k(&decoded) {
                        let output_line = format_n2k_output(pgn, src, &data);
                        if let Err(e) = writeln!(output, "{}", output_line) {
                            warn!("Failed to write output: {e}");
                            return;
                        }
                    }
                }
                Err(e) => {
                    warn!("Decode error: {e}");
                }
            }
            
            bytes.drain(0..frame_len);
        }
    }
}

fn process_serial_input(input: &mut Box<dyn Read>, output: &mut Box<dyn Write>) {
    let mut buffer = [0u8; 1024];
    let mut remaining = Vec::new();
    
    loop {
        match input.read(&mut buffer) {
            Ok(0) => {
                info!("End of input stream");
                break;
            }
            Ok(n) => {
                remaining.extend_from_slice(&buffer[..n]);
                
                while remaining.len() >= 6 {
                    let body_size = remaining[2] as usize;
                    let frame_len = 5 + body_size + 1;
                    
                    if remaining.len() < frame_len {
                        break;
                    }
                    
                    let frame = &remaining[0..frame_len];
                    match decode_frame(frame) {
                        Ok(decoded) => {
                            info!("Decoded: {} -> {:?}", decoded.command, decoded.values);
                            if let Some((pgn, src, data)) = decode_frame_to_n2k(&decoded) {
                                let output_line = format_n2k_output(pgn, src, &data);
                                if let Err(e) = writeln!(output, "{}", output_line) {
                                    warn!("Failed to write output: {e}");
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Decode error: {e}");
                        }
                    }
                    
                    remaining.drain(0..frame_len);
                }
            }
            Err(e) => {
                error!("Read error: {e}");
                break;
            }
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input> <output>", args[0]);
        eprintln!("  input:  file path or serial port (e.g., /dev/ttyUSB0)");
        eprintln!("  output: can0, stdout, or file path");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    info!("Opening input: {}", input_path);
    let input = match open_input(input_path) {
        Ok(i) => i,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    };

    info!("Opening output: {}", output_path);
    let mut output = match open_output(output_path) {
        Ok(o) => o,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    };

    if Path::new(input_path).exists() {
        process_file_input(input_path, &mut output);
    } else {
        let mut input = input;
        process_serial_input(&mut input, &mut output);
    }
}