# Fastnet to NMEA2000 Mapping

This document describes how Fastnet protocol channels are mapped to NMEA2000 PGNs via Signal K paths.

## Architecture

```
B&G Fastnet → bgfastnet_lib (Signal K) → fastnet2n2k (NMEA2000 PGNs)
```

1. **bgfastnet_lib** (Rust) decodes Fastnet frames to Signal K paths in SI units
2. **fastnet2n2k** (Rust) converts Signal K paths to NMEA2000 PGNs

The mapping is identical to pyfastnet's `signalk_map.py`.

## Signal K Path → NMEA2000 PGN Mapping

| Signal K Path | NMEA2000 PGN | PGN Name | Output Fields |
|---------------|--------------|----------|---------------|
| navigation.attitude.roll | 127240 | Attitude | Roll (f32, rad) |
| navigation.attitude.pitch | 127240 | Attitude | Pitch (f32, rad) |
| navigation.rateOfTurn | 127245 | Rate of Turn | Rate of Turn (f32, rad/s) |
| navigation.leewayAngle | 127249 | Leeway | Leeway Angle (f32, rad) |
| navigation.headingTrue/navigation.headingMagnetic | 127250 | Heading | Heading (f32, rad) |
| navigation.headingTrue/navigation.headingMagnetic | 127251 | Vessel Heading | Heading (f32, rad) + Rudder Angle |
| steering.rudderAngle | 127251 | Vessel Heading | Rudder Angle (f32, rad) |
| navigation.speedThroughWater | 127257 | Speed (Boat Speed) | Speed (f32, m/s) |
| environment.wind.speedApparent | 128258 | Apparent Wind Speed | Wind Speed (f32, m/s) |
| environment.wind.speedTrue | 128259 | True Wind Speed | Wind Speed (f32, m/s) |
| environment.depth.belowTransducer | 128267 | Water Depth | Depth (f32, m) |
| navigation.log | 128275 | Distance | Distance (f32, m) |
| navigation.position | 129025 | Position (Lat) | Latitude (f64, deg) |
| navigation.speedOverGround | 129291 | COG & SOG | SOG (f32, knots) |
| navigation.courseOverGroundTrue | 129291 | COG & SOG | COG (f32, rad) |

## Complete Signal K Path Map (from bgfastnet_lib/pyfastnet)

The library supports these Signal K paths (not all have NMEA2000 equivalents):

### Navigation
- navigation.speedThroughWater (0x41) → 127257
- navigation.speedOverGround (0xEB) → 129291
- navigation.courseOverGroundTrue (0xE9) → 129291
- navigation.courseOverGroundMagnetic (0xEA)
- navigation.rateOfTurn (0x44) → 127245
- navigation.attitude.roll (0x34) → 127240
- navigation.attitude.pitch (0x9B) → 127240
- navigation.leewayAngle (0x82) → 127249
- navigation.log (0xCD) → 128275
- navigation.trip.log (0xCF)
- navigation.courseRhumbline.nextPoint.distance (0xE7)
- navigation.courseGreatCircle.nextPoint.distance (0xE8, 0xFA)
- navigation.courseRhumbline.nextPoint.bearingTrue (0xE3)
- navigation.courseRhumbline.nextPoint.bearingMagnetic (0xE4)
- navigation.courseGreatCircle.nextPoint.bearingTrue (0xE5)
- navigation.courseGreatCircle.nextPoint.bearingMagnetic (0xE6)
- navigation.courseGreatCircle.bearingTrackTrue (0xE0)
- navigation.courseGreatCircle.bearingTrackMagnetic (0xE1)
- navigation.courseGreatCircle.nextPoint.velocityMadeGood (0xEC)
- navigation.courseGreatCircle.nextPoint.timeToGo (0xED)
- navigation.courseGreatCircle.crossTrackError (0xEE)
- navigation.racing.layline.distance (0xE2)
- navigation.racing.layline.time (0xFB)

### Environment
- environment.water.temperature (0x1F)
- environment.outside.temperature (0x1D)
- environment.outside.pressure (0x87)
- environment.wind.speedApparent (0x4F) → 128258
- environment.wind.angleApparent (0x51)
- environment.wind.speedTrue (0x56) → 128259
- environment.wind.angleTrueWater (0x59)
- environment.current.drift (0x83)

### Steering
- steering.rudderAngle (0x0B) → 127251
- steering.autopilot.target.headingMagnetic (0xA6)
- steering.autopilot.state (0xB5)

### Performance
- performance.velocityMadeGood (0x7F)
- performance.targetSpeed (0x7D)
- performance.targetAngle (0x53)
- performance.polarSpeedRatio (0x7C)
- performance.tackMagnetic (0x9A)

### Electrical
- electrical.batteries.{id}.voltage (0x8D)

### Vendor (bandg.*)
- bandg.wind.measuredSpeed (0x57)
- bandg.wind.measuredAngle (0x5A)
- bandg.wind.upwash (0x85)
- bandg.wind.rawSpeedApparent (0x4E)
- bandg.wind.rawAngleApparent (0x52)
- bandg.navigation.rawSpeedThroughWater (0x42)
- bandg.navigation.rawHeading (0x4A)
- bandg.mast.rotation (0x9C)
- bandg.mast.windAngle (0x9D)
- bandg.performance.headLiftTrend (0x27)
- bandg.performance.tacking (0x32)
- bandg.performance.reaching (0x33)
- bandg.performance.courseToSail (0xF9)
- bandg.performance.optimumWindAngle (0x35)
- bandg.performance.nextLeg.angleApparent (0x6F)
- bandg.performance.nextLeg.speedApparent (0x71)
- bandg.performance.nextLeg.targetSpeed (0x70)
- bandg.navigation.speedThroughWaterAverage (0x64)
- bandg.navigation.courseThroughWater (0x69)
- bandg.navigation.deadReckoning.distance (0x81)
- bandg.navigation.deadReckoning.course (0xD3)
- bandg.motion.rate (0x3C)
- bandg.motion.pitchRate (0x9E)
- bandg.steering.offCourse (0x29)
- bandg.steering.autopilot.offCourse (0xAF)
- bandg.steering.autopilot.fixedSpeed (0x46)
- bandg.environment.pressureTrend (0x86)
- bandg.time.local (0xDC)
- bandg.time.timer (0x75)

## Notes

- Timestamps are in ISO 8601 format with "Z" suffix (UTC)
- Angles are in radians
- Speeds: knots for SOG/boatspeed, m/s for wind
- Distance is in meters (NMEA2000 standard)
- All values are written as little-endian floats/bytes in the PGN payload

## Unsupported PGNs from Requested List

The following requested PGNs are not supported because the bgfastnet_lib/pyfastnet library does not provide the necessary Signal K paths:

- 59392, 59904, 60928 (ISO/proprietary frames)
- 126992-126998 (System/Product/Identity information)
- 127258 (Magnetic Variation)
- 127508 (Rudder - different PGN)
- 128000 (Speed - different PGN)
- 129026, 129029 (Position - uses same Signal K path as 129025)
- 129539, 129540 (AIS data)
- 130306, 130312 (Wind Speed - different PGNs)