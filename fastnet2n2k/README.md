# fastnet2n2k

Convert FastNet protocol to NMEA2000 PGNs.

## Usage

```
fastnet2n2k <input> <output>

Arguments:
  input   Input source: file path or serial port (e.g., /dev/ttyUSB0)
  output  Output destination: file path, or "-" for stdout (PLAIN format)

Supported PGNs:
  127240 - Attitude (roll, pitch)
  127245 - Rate of Turn
  127249 - Leeway
  127250 - Heading
  127251 - Vessel Heading
  127257 - Speed (Boat Speed)
  128258 - Apparent Wind Speed
  128259 - True Wind Speed
  128267 - Water Depth
  128275 - Distance
  129025 - Position (Latitude)
  129291 - COG & SOG
```

## Examples

```bash
# From serial port to file (PLAIN format)
fastnet2n2k /dev/ttyUSB0 output.txt

# From file to stdout
fastnet2n2k input.txt -

# From serial port to file
fastnet2n2k /dev/ttyUSB0 output.txt
```

## Output format (PLAIN)

Output follows canboat PLAIN format:
```
2026-08-05T05:32:36.387Z,3,128258,0,CD,CC,04,41,00,00,00,00
```

Fields: timestamp, priority, PGN, source, data bytes...

## Signal K → PGN Mapping

See `MAPPING.md` for complete Signal K path to NMEA2000 PGN mappings.