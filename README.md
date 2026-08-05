# rust-bgfastnet

A Rust workspace for decoding the FastNet protocol used by B&G Hydra / H2000 marine instruments.

## Overview

This project provides tools for working with the FastNet protocol:

- **bgfastnet_lib**: Library crate for decoding FastNet protocol frames
- **fastnet-processor**: CLI tool for processing FastNet data from files or serial ports
- **fastnet2n2k**: Converts FastNet data to NMEA2000 PGNs for Signal K and marine networks

## What is FastNet?

FastNet is B&G's proprietary marine instrument protocol used by their Hydra / H2000 systems. It encodes navigation, wind, autopilot, and environmental data from connected sensors.

## Features

- Decode FastNet protocol frames (standard, ASCII, light)
- Convert to Signal K paths with SI unit conversion
- Support for serial port and file input
- Comprehensive test coverage
- NMEA2000 PGN mapping for marine network integration

## Workspace Structure

```
rust-bgfastnet/
├── bgfastnet_lib/          # Library crate
│   └── src/
├── fastnet-processor/      # CLI tool
│   └── src/
│       └── test-fastnet-data.txt
├── fastnet2n2k/            # FastNet to NMEA2000 converter
│   └── src/
└── README.md              # This file
```

## Quick Start

### As a library

```toml
[dependencies]
bgfastnet_lib = "0.1.0"
```

```rust
use bgfastnet_lib::{FrameBuffer, decode_frame};

let mut fb = FrameBuffer::new_default();
fb.add_to_buffer(&raw_data);
fb.get_complete_frames();

while let Some(frame) = fb.frame_queue_pop() {
    for (path, value) in &frame.values {
        println!("{} = {}", path, value);
    }
}
```

### As a CLI tool

```bash
# From a hex-encoded file
fastnet-processor data.hex

# From a serial port (fixed at 28800 8O2)
fastnet-processor /dev/ttyUSB0
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test --package bgfastnet_lib --package fastnet2n2k
```

## License

MIT