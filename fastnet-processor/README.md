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
fastnet-processor --file data.hex
```

### From serial port

```bash
fastnet-processor --serial /dev/ttyUSB0 --baud 38400
```

### Output format

```
navigation.speedThroughWater = 2.5
navigation.attitude.roll = 0.123
environment.wind.speedApparent = 5.7
```

## Examples

See the `examples/` directory for sample FastNet data files.

## License

MIT