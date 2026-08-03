# bgfastnet_lib

A Rust library for decoding the FastNet protocol used by B&G Hydra / H2000 marine instruments.

[![Crates.io](https://img.shields.io/crates/v/bgfastnet_lib)](https://crates.io/crates/bgfastnet_lib)
[![Documentation](https://docs.rs/bgfastnet_lib/badge.svg)](https://docs.rs/bgfastnet_lib)

## Overview

This library decodes FastNet protocol data streams from B&G instruments and converts them to Signal K paths in SI units.

## Quick Start

```rust
use bgfastnet_lib::{FrameBuffer, decode_frame};

let mut fb = FrameBuffer::new_default();

// Feed raw bytes from the FastNet bus
let data = vec![];
fb.add_to_buffer(&data);
fb.get_complete_frames();

// Get decoded frames
while let Some(frame) = fb.frame_queue_pop() {
    for (path, value) in &frame.values {
        println!("{} = {}", path, value);
    }
}
```

## Detailed Usage

### 1. Reading Raw Data

Feed raw bytes from the FastNet bus into the `FrameBuffer`:

```rust
use bgfastnet_lib::FrameBuffer;

let mut fb = FrameBuffer::new_default();
let raw_data: Vec<u8> = vec![0xFF, 0x05, 0x04, 0x01, 0xE7, 0x4F, 0x61, 0x00, 0x50, 0x9B];
fb.add_to_buffer(&raw_data);
```

### 2. Processing Complete Frames

Call `get_complete_frames()` to parse and decode frames from the buffer:

```rust
fb.get_complete_frames();
```

### 3. Retrieving Decoded Data

Pop decoded frames from the queue:

```rust
while let Some(frame) = fb.frame_queue_pop() {
    println!("Command: {}", frame.command);
    println!("From: {}", frame.from_address);
    
    for (path, value) in &frame.values {
        println!("  {} = {}", path, value);
    }
}
```

### 4. Direct Frame Decoding

For manual frame decoding:

```rust
use bgfastnet_lib::{decode_frame, decode_ascii_frame, decode_light_frame};

// Standard broadcast frames
let frame = vec![0xFF, 0x05, 0x04, 0x01, 0xE7, 0x4F, 0x61, 0x00, 0x50, 0x9B];
let decoded = decode_frame(&frame)?;

// ASCII frames (e.g., LatLon position)
let ascii_frame = vec![0xFF, 0x05, 0x0E, 0x03, 0x03, 0x47, 0x00, 0x33, 0x33, 0x35, 0x32, 0x2E, 0x34, 0x35, 0x30, 0x53, 0x31, 0x35, 0x31, 0x31, 0x33, 0x2E, 0x39, 0x32, 0x30, 0x45, 0x9B];
let decoded = decode_ascii_frame(&ascii_frame)?;

// Light intensity frames
let light_frame = vec![0xFF, 0x50, 0x01, 0xC9, 0xE7, 0x04, 0xFC];
let decoded = decode_light_frame(&light_frame)?;
```

### 5. Accessing Raw Decoded Values

For more details about decoded values:

```rust
use bgfastnet_lib::decode_frame;

let frame = vec![0xFF, 0x05, 0x04, 0x01, 0xE7, 0x4F, 0x61, 0x00, 0x50, 0x9B];
let decoded = decode_frame(&frame)?;

for (channel_name, value_info) in &decoded.values {
    println!("Channel: {}", channel_name);
    println!("  Raw value: {:?}", value_info.value);
    println!("  Display text: {:?}", value_info.display_text);
    println!("  Layout: {:?}", value_info.layout);
}
```

## Supported Channels

The library supports decoding of all FastNet channels including:

- **Navigation**: Speed, heading, course, position, depth
- **Wind**: Apparent, true, and magnetic wind data
- **Autopilot**: Mode, target, rudder angle
- **Performance**: VMG, polar ratios, tacking metrics
- **Environment**: Temperature, pressure, current
- **Vendor-specific**: B&G extended channels (bandg.*)

## Unit Conversion

All values are automatically converted to SI units:

| Path | Unit |
|------|------|
| `navigation.speedThroughWater` | m/s |
| `navigation.attitude.roll` | rad |
| `environment.wind.speedApparent` | m/s |
| `environment.water.temperature` | K |
| `environment.depth.belowTransducer` | m |

## Signal K Mapping

When `project` is enabled (default), frames are mapped to Signal K paths:

```rust
let fb = FrameBuffer::new_default(); // project=true by default
// Maps to Signal K paths automatically

let fb = FrameBuffer::new(8192, 1000, false); // Disable projection
// Returns raw channel values
```

## Error Handling

Decoding functions return `Result<DecodedFrame, String>`:

```rust
match decode_frame(&frame) {
    Ok(decoded) => {
        // Process decoded data
    }
    Err(e) => {
        eprintln!("Decoding error: {}", e);
    }
}
```

## Features

- Decode FastNet protocol frames
- Convert to Signal K paths
- SI unit conversion
- Support for ASCII and light frames
- Comprehensive test coverage

## License

MIT