use bgfastnet_lib::FrameBuffer;
use serialport::{DataBits, Parity, StopBits};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::time::Duration;

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim().replace([' ', '\n', '\t', '\r'], "");
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn process_frame_queue(fb: &mut FrameBuffer) {
    fb.get_complete_frames();

    while let Some(frame) = fb.frame_queue_pop() {
        println!(
            "[{}] {} -> {}",
            frame.command, frame.from_address, frame.to_address
        );
        for (path, value) in &frame.values {
            println!("  {} = {}", path, value);
        }
        println!();
    }
}

fn read_from_file(path: &str) {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut fb = FrameBuffer::new_default();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let bytes = hex_to_bytes(&line);
        if !bytes.is_empty() {
            fb.add_to_buffer(&bytes);
            process_frame_queue(&mut fb);
        }
    }
}

fn read_from_serial(port: &str) {
    let mut serial = serialport::new(port, 28800)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::Two)
        .parity(Parity::Odd)
        .timeout(Duration::from_secs(1))
        .open()
        .expect("Failed to open serial port");

    let mut fb = FrameBuffer::new_default();
    let mut buffer = [0u8; 1024];

    loop {
        match serial.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                fb.add_to_buffer(&buffer[..n]);
                process_frame_queue(&mut fb);
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => {
                eprintln!("Error reading serial port: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file|serial_port>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    if Path::new(input_path).exists() {
        read_from_file(input_path);
    } else {
        read_from_serial(input_path);
    }
}
