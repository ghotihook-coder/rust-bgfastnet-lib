# fastnet-processor

A CLI tool for processing FastNet protocol data from B&G instruments.

## Overview

The `fastnet-processor` tool reads hex-encoded FastNet data from files or serial ports and outputs decoded Signal K paths.

## Installation

```bash
cargo install --path .
```

## Usage

### From a file

```bash
fastnet-processor data.hex
```

### From serial port

```bash
fastnet-processor /dev/ttyUSB0
```

The serial port uses non-standard configuration:
- Baud rate: 28800
- Data bits: 8
- Stop bits: 2
- Parity: Odd

### Output format

```
navigation.speedThroughWater = 2.5
navigation.attitude.roll = 0.123
environment.wind.speedApparent = 5.7
```

## Examples

See `test-fastnet-data.txt` in this directory for sample FastNet data.

## License

MIT