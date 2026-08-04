# rust-bgfastnet

A Rust workspace for decoding the FastNet protocol used by B&G Hydra / H2000 marine instruments.

## Overview

This project provides tools for working with the FastNet protocol:

- **bgfastnet_lib**: Library crate for decoding FastNet protocol frames
- **fastnet-processor**: CLI tool for processing FastNet data from files or serial ports

## What is FastNet?

FastNet is B&G's proprietary marine instrument protocol used by their Hydra / H2000 systems. It encodes navigation, wind, autopilot, and environmental data from connected sensors.

## Features

- Decode FastNet protocol frames (standard, ASCII, light)
- Convert to Signal K paths with SI unit conversion
- Support for serial port and file input
- Comprehensive test coverage

## Workspace Structure

```
rust-bgfastnet/
├── bgfastnet_lib/          # Library crate
│   └── src/
├── fastnet-processor/      # CLI tool
│   └── src/
│       └── test-fastnet-data.txt
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
# From a hex file
fastnet-processor --file data.hex

# From serial port
fastnet-processor --serial /dev/ttyUSB0 --baud 38400
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

MIT