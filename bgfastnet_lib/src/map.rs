use std::collections::HashMap;
use std::f64::consts::PI;

const KN_MS: f64 = 0.514444;
const NM_M: f64 = 1852.0;
const FT_M: f64 = 0.3048;
const FATHOM_M: f64 = 1.8288;
const DEG_RAD: f64 = PI / 180.0;
const HPA_PA: f64 = 100.0;

fn kn(v: f64) -> f64 {
    v * KN_MS
}

fn nm(v: f64) -> f64 {
    v * NM_M
}

fn deg(v: f64) -> f64 {
    v * DEG_RAD
}

fn c2k(v: f64) -> f64 {
    v + 273.15
}

fn hpa(v: f64) -> f64 {
    v * HPA_PA
}

fn pct(v: f64) -> f64 {
    v / 100.0
}

fn id(v: f64) -> f64 {
    v
}

type Transform = fn(f64) -> f64;

const STANDARD: &[(&str, u8, Transform)] = &[
    ("navigation.speedThroughWater", 0x41, kn),
    ("navigation.speedOverGround", 0xEB, kn),
    ("navigation.courseOverGroundTrue", 0xE9, deg),
    ("navigation.courseOverGroundMagnetic", 0xEA, deg),
    ("navigation.rateOfTurn", 0x44, deg),
    ("navigation.attitude.roll", 0x34, deg),
    ("navigation.attitude.pitch", 0x9B, deg),
    ("navigation.leewayAngle", 0x82, deg),
    ("navigation.log", 0xCD, nm),
    ("navigation.trip.log", 0xCF, nm),
    ("navigation.courseRhumbline.nextPoint.distance", 0xE7, nm),
    ("navigation.courseGreatCircle.nextPoint.distance", 0xE8, nm),
    ("navigation.courseGreatCircle.nextPoint.distance", 0xFA, nm),
    ("navigation.courseRhumbline.nextPoint.bearingTrue", 0xE3, deg),
    ("navigation.courseRhumbline.nextPoint.bearingMagnetic", 0xE4, deg),
    ("navigation.courseGreatCircle.nextPoint.bearingTrue", 0xE5, deg),
    ("navigation.courseGreatCircle.nextPoint.bearingMagnetic", 0xE6, deg),
    ("navigation.courseGreatCircle.bearingTrackTrue", 0xE0, deg),
    ("navigation.courseGreatCircle.bearingTrackMagnetic", 0xE1, deg),
    ("navigation.courseGreatCircle.nextPoint.velocityMadeGood", 0xEC, kn),
    ("navigation.courseGreatCircle.nextPoint.timeToGo", 0xED, id),
    ("navigation.courseGreatCircle.crossTrackError", 0xEE, nm),
    ("navigation.racing.layline.distance", 0xE2, nm),
    ("navigation.racing.layline.time", 0xFB, id),
    ("environment.water.temperature", 0x1F, c2k),
    ("environment.outside.temperature", 0x1D, c2k),
    ("environment.outside.pressure", 0x87, hpa),
    ("environment.wind.speedApparent", 0x4F, id),
    ("environment.wind.angleApparent", 0x51, deg),
    ("environment.wind.speedTrue", 0x56, id),
    ("environment.wind.angleTrueWater", 0x59, deg),
    ("environment.current.drift", 0x83, kn),
    ("steering.rudderAngle", 0x0B, deg),
    ("steering.autopilot.target.headingMagnetic", 0xA6, deg),
    ("performance.velocityMadeGood", 0x7F, kn),
    ("performance.targetSpeed", 0x7D, kn),
    ("performance.targetAngle", 0x53, deg),
    ("performance.polarSpeedRatio", 0x7C, pct),
    ("performance.tackMagnetic", 0x9A, deg),
    ("electrical.batteries.house.voltage", 0x8D, id),
];

const DEPTH: &[(u8, Transform)] = &[(0xC1, id), (0xC2, |v| v * FT_M), (0xC3, |v| v * FATHOM_M)];
const DEPTH_PATH: &str = "environment.depth.belowTransducer";

const ROUTED: &[(&str, &str, u8, Transform)] = &[
    ("navigation.headingMagnetic", "navigation.headingTrue", 0x49, deg),
    ("environment.wind.directionMagnetic", "environment.wind.directionTrue", 0x6D, deg),
    ("environment.current.setMagnetic", "environment.current.setTrue", 0x84, deg),
];

const VENDOR: &[(&str, u8, Transform)] = &[
    ("bandg.wind.measuredSpeed", 0x57, kn),
    ("bandg.wind.measuredAngle", 0x5A, deg),
    ("bandg.wind.upwash", 0x85, deg),
    ("bandg.wind.rawSpeedApparent", 0x4E, id),
    ("bandg.wind.rawAngleApparent", 0x52, id),
    ("bandg.navigation.rawSpeedThroughWater", 0x42, id),
    ("bandg.navigation.rawHeading", 0x4A, id),
    ("bandg.mast.rotation", 0x9C, deg),
    ("bandg.mast.windAngle", 0x9D, deg),
    ("bandg.performance.headLiftTrend", 0x27, id),
    ("bandg.performance.tacking", 0x32, pct),
    ("bandg.performance.reaching", 0x33, pct),
    ("bandg.performance.courseToSail", 0xF9, deg),
    ("bandg.performance.optimumWindAngle", 0x35, deg),
    ("bandg.performance.nextLeg.angleApparent", 0x6F, deg),
    ("bandg.performance.nextLeg.speedApparent", 0x71, kn),
    ("bandg.performance.nextLeg.targetSpeed", 0x70, kn),
    ("bandg.navigation.speedThroughWaterAverage", 0x64, kn),
    ("bandg.navigation.courseThroughWater", 0x69, deg),
    ("bandg.navigation.deadReckoning.distance", 0x81, nm),
    ("bandg.navigation.deadReckoning.course", 0xD3, deg),
    ("bandg.motion.rate", 0x3C, id),
    ("bandg.motion.pitchRate", 0x9E, deg),
    ("bandg.steering.offCourse", 0x29, deg),
    ("bandg.steering.autopilot.offCourse", 0xAF, deg),
    ("bandg.steering.autopilot.fixedSpeed", 0x46, kn),
    ("bandg.environment.pressureTrend", 0x86, id),
    ("bandg.time.local", 0xDC, id),
    ("bandg.time.timer", 0x75, id),
];

const COLLAPSED: &[u8] = &[0x1C, 0x1E, 0x4D, 0x55, 0x65];
const DROP: &[u8] = &[0x00, 0x50, 0x68, 0x6A, 0x36, 0x37, 0xC9];

const MAGNETIC_LAYOUT: &str = "°M";

#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub name: String,
    pub path: String,
    pub unit: String,
    pub kind: String,
}

pub fn project(decoded_frame: &crate::decode::DecodedFrame) -> HashMap<String, f64> {
    let mut out: HashMap<String, f64> = HashMap::new();
    let command = &decoded_frame.command;
    let values = &decoded_frame.values;

    if command == "LatLon" {
        for entry in values.values() {
            if let Some(display_text) = &entry.display_text {
                if let Some(pos) = parse_position(display_text) {
                    out.insert("navigation.position".to_string(), pos);
                    return out;
                }
            }
        }
        return out;
    }

    let mut depth_seen: HashMap<u8, f64> = HashMap::new();
    for (_channel_name, entry) in values {
        let cid = parse_channel_id(&entry.channel_id);
        
        if let Some(cid) = cid {
            if DROP.contains(&cid) || COLLAPSED.contains(&cid) {
                continue;
            }

            let value = entry.value;

            if cid == 0xC1 || cid == 0xC2 || cid == 0xC3 {
                if let Some(v) = value {
                    depth_seen.insert(cid, v);
                }
                continue;
            }

            if cid == 0xB5 {
                if let Some(state) = ap_state(value) {
                    let state_val = match state {
                        "standby" => 0.0,
                        "auto" => 1.0,
                        "wind" => 2.0,
                        "route" => 3.0,
                        "directControl" => 4.0,
                        _ => 0.0,
                    };
                    out.insert("steering.autopilot.state".to_string(), state_val);
                }
                continue;
            }

            if let Some((mag_path, true_path, _, tf)) = ROUTED.iter().find(|(_, _, c, _)| *c == cid) {
                let layout = entry.layout.as_deref();
                let path = if layout == Some(MAGNETIC_LAYOUT) { mag_path } else { true_path };
                if let Some(v) = value {
                    out.insert(path.to_string(), tf(v));
                }
                continue;
            }

            if let Some((path, _, tf)) = STANDARD.iter().find(|(_, c, _)| *c == cid) {
                let path = if path.contains("{id}") {
                    path.replace("{id}", "house")
                } else {
                    path.to_string()
                };
                if let Some(v) = value {
                    out.insert(path, tf(v));
                }
                continue;
            }

            if let Some((path, _, tf)) = VENDOR.iter().find(|(_, c, _)| *c == cid) {
                if let Some(v) = value {
                    out.insert(path.to_string(), tf(v));
                }
                continue;
            }

            if let Some(v) = value {
                out.insert(format!("bandg.unknown.0x{:02X}", cid), v);
            }
        }
    }

    for (cid, tf) in DEPTH {
        if let Some(v) = depth_seen.get(cid) {
            out.insert(DEPTH_PATH.to_string(), tf(*v));
            break;
        }
    }

    out
}

fn parse_channel_id(channel_id: &str) -> Option<u8> {
    let raw = channel_id.strip_prefix("0x").or(channel_id.strip_prefix("0X"));
    raw.and_then(|s| u8::from_str_radix(s, 16).ok())
}

fn ap_state(value: Option<f64>) -> Option<&'static str> {
    let raw = value?.round() as i32;
    let high = (raw >> 8) & 0xFF;
    let low = (raw & 0xFF) as u8;
    
    if high == 0x50 {
        return Some("standby");
    }
    if high == 0x51 || high == 0x59 {
        let autopilot_mode_by_low: HashMap<u8, &str> = crate::constants::AUTOPILOT_MODE_BY_LOW.iter().cloned().collect();
        return autopilot_mode_by_low.get(&low).map(|s| *s);
    }
    None
}

pub fn parse_position(ascii_text: &str) -> Option<f64> {
    let ascii_text = ascii_text.trim();
    
    let lat_n = ascii_text.find('N').unwrap_or(0);
    let lat_s = ascii_text.find('S').unwrap_or(0);
    let lat_i = std::cmp::max(lat_n, lat_s);
    
    let lon_e = ascii_text.find('E').unwrap_or(0);
    let lon_w = ascii_text.find('W').unwrap_or(0);
    let lon_i = std::cmp::max(lon_e, lon_w);
    
    if lat_i == 0 || lon_i == 0 || lat_i >= ascii_text.len() - 1 || lon_i <= lat_i {
        return None;
    }

    let (lat_part, lat_dir) = ascii_text.split_at(lat_i);
    let (lon_part, lon_dir) = ascii_text[lat_i + 1..].split_at(lon_i - lat_i - 1);

    let lat_deg: u32 = lat_part[..2].parse().ok()?;
    let lat_min: f64 = lat_part[2..].parse().ok()?;
    let lat = lat_deg as f64 + lat_min / 60.0;
    let lat = if lat_dir == "S" { -lat } else { lat };

    let lon_deg: u32 = lon_part[..3].parse().ok()?;
    let lon_min: f64 = lon_part[3..].parse().ok()?;
    let lon = lon_deg as f64 + lon_min / 60.0;
    let lon = if lon_dir == "W" { -lon } else { lon };

    // Encode as a single f64 for simplicity (lat + lon offset)
    // In practice, this would be a struct {latitude, longitude}
    Some(lat + lon / 1000.0)
}

pub fn unit_for(path: &str) -> &'static str {
    match path {
        "navigation.speedThroughWater" => "m/s",
        "navigation.speedOverGround" => "m/s",
        "navigation.courseOverGroundTrue" => "rad",
        "navigation.courseOverGroundMagnetic" => "rad",
        "navigation.rateOfTurn" => "rad",
        "navigation.attitude.roll" => "rad",
        "navigation.attitude.pitch" => "rad",
        "navigation.leewayAngle" => "rad",
        "navigation.log" => "m",
        "navigation.trip.log" => "m",
        "navigation.courseRhumbline.nextPoint.distance" => "m",
        "navigation.courseGreatCircle.nextPoint.distance" => "m",
        "navigation.courseRhumbline.nextPoint.bearingTrue" => "rad",
        "navigation.courseRhumbline.nextPoint.bearingMagnetic" => "rad",
        "navigation.courseGreatCircle.nextPoint.bearingTrue" => "rad",
        "navigation.courseGreatCircle.nextPoint.bearingMagnetic" => "rad",
        "navigation.courseGreatCircle.bearingTrackTrue" => "rad",
        "navigation.courseGreatCircle.bearingTrackMagnetic" => "rad",
        "navigation.courseGreatCircle.nextPoint.velocityMadeGood" => "m/s",
        "navigation.courseGreatCircle.nextPoint.timeToGo" => "s",
        "navigation.courseGreatCircle.crossTrackError" => "m",
        "navigation.racing.layline.distance" => "m",
        "navigation.racing.layline.time" => "s",
        "environment.water.temperature" => "K",
        "environment.outside.temperature" => "K",
        "environment.outside.pressure" => "Pa",
        "environment.wind.speedApparent" => "m/s",
        "environment.wind.angleApparent" => "rad",
        "environment.wind.speedTrue" => "m/s",
        "environment.wind.angleTrueWater" => "rad",
        "environment.current.drift" => "m/s",
        "steering.rudderAngle" => "rad",
        "steering.autopilot.target.headingMagnetic" => "rad",
        "performance.velocityMadeGood" => "m/s",
        "performance.targetSpeed" => "m/s",
        "performance.targetAngle" => "rad",
        "performance.polarSpeedRatio" => "ratio",
        "performance.tackMagnetic" => "rad",
        "electrical.batteries.house.voltage" => "V",
        "environment.depth.belowTransducer" => "m",
        "steering.autopilot.state" => "enum",
        _ => "",
    }
}

pub fn channel_map() -> Vec<ChannelInfo> {
    let channel_lookup: HashMap<u8, &str> = crate::constants::CHANNEL_LOOKUP.iter().map(|(s, v)| (*v, *s)).collect();
    let mut out = Vec::new();

    for (cid, name) in channel_lookup.iter() {
        let info = channel_disposition(*cid, name);
        out.push(info);
    }

    out
}

fn channel_disposition(cid: u8, name: &str) -> ChannelInfo {
    if ROUTED.iter().any(|(_, _, c, _)| *c == cid) {
        let stem = ROUTED.iter().find(|(_, _, c, _)| *c == cid).map(|(m, _, _, _)| {
            m.strip_suffix("Magnetic").unwrap_or(m)
        }).unwrap_or("");
        return ChannelInfo {
            name: name.to_string(),
            path: format!("{}{{Magnetic,True}}", stem),
            unit: "rad".to_string(),
            kind: "routed(M/T)".to_string(),
        };
    }

    if cid == 0xC1 || cid == 0xC2 || cid == 0xC3 {
        return ChannelInfo {
            name: name.to_string(),
            path: DEPTH_PATH.to_string(),
            unit: "m".to_string(),
            kind: "standard(depth fallback)".to_string(),
        };
    }

    if cid == 0xB5 {
        return ChannelInfo {
            name: name.to_string(),
            path: "steering.autopilot.state".to_string(),
            unit: "enum".to_string(),
            kind: "standard".to_string(),
        };
    }

    if let Some((path, _, _)) = STANDARD.iter().find(|(_, c, _)| *c == cid) {
        let path = if path.contains("{id}") {
            path.replace("{id}", "house")
        } else {
            path.to_string()
        };
        let path_clone = path.clone();
        return ChannelInfo {
            name: name.to_string(),
            path,
            unit: unit_for(&path_clone).to_string(),
            kind: "standard".to_string(),
        };
    }

    if let Some((path, _, _)) = VENDOR.iter().find(|(_, c, _)| *c == cid) {
        return ChannelInfo {
            name: name.to_string(),
            path: path.to_string(),
            unit: unit_for(path).to_string(),
            kind: "vendor".to_string(),
        };
    }

    if COLLAPSED.contains(&cid) {
        let collapsed: HashMap<u8, &str> = [(0x1C, "environment.outside.temperature"), (0x1E, "environment.water.temperature"), (0x4D, "environment.wind.speedApparent"), (0x55, "environment.wind.speedTrue"), (0x65, "bandg.navigation.speedThroughWaterAverage")].into_iter().collect();
        return ChannelInfo {
            name: name.to_string(),
            path: format!("→ {}", collapsed.get(&cid).unwrap()),
            unit: "".to_string(),
            kind: "collapsed".to_string(),
        };
    }

    if DROP.contains(&cid) {
        return ChannelInfo {
            name: name.to_string(),
            path: "—".to_string(),
            unit: "".to_string(),
            kind: "drop".to_string(),
        };
    }

    ChannelInfo {
        name: name.to_string(),
        path: format!("bandg.unknown.0x{:02X}", cid),
        unit: "".to_string(),
        kind: "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_basic() {
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Wind Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("Apparent Wind Speed (m/s)".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x4F".to_string(),
                    value: Some(3.8),
                    display_text: Some("3.8".to_string()),
                    layout: None,
                }),
                ("Apparent Wind Angle".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x51".to_string(),
                    value: Some(45.0),
                    display_text: Some("45.0".to_string()),
                    layout: None,
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("environment.wind.speedApparent"));
        assert!(result.contains_key("environment.wind.angleApparent"));
    }

    #[test]
    fn test_parse_position() {
        let result = parse_position("3352.450S15113.920E");
        assert!(result.is_some());
    }

    #[test]
    fn test_unit_for() {
        assert_eq!(unit_for("navigation.speedThroughWater"), "m/s");
        assert_eq!(unit_for("environment.wind.angleApparent"), "rad");
    }

    #[test]
    fn test_project_autopilot_state() {
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Pilot FFD (50)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("Autopilot Mode".to_string(), crate::decode::DecodedValue {
                    channel_id: "0xB5".to_string(),
                    value: Some(20480.0), // 0x5000 = standby
                    display_text: Some("Standby".to_string()),
                    layout: None,
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("steering.autopilot.state"));
    }

    #[test]
    fn test_project_twd_magnetic() {
        // True Wind Direction with magnetic layout (channel 0x6D)
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Wind Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("True Wind Direction".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x6D".to_string(),
                    value: Some(180.0),
                    display_text: Some("180.0°M".to_string()),
                    layout: Some("°M".to_string()),
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("environment.wind.directionMagnetic"));
        assert!(!result.contains_key("environment.wind.directionTrue"));
    }

    #[test]
    fn test_project_twd_true() {
        // True Wind Direction with true layout (channel 0x6D)
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Wind Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("True Wind Direction".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x6D".to_string(),
                    value: Some(180.0),
                    display_text: Some("180.0°T".to_string()),
                    layout: Some("°T".to_string()),
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("environment.wind.directionTrue"));
        assert!(!result.contains_key("environment.wind.directionMagnetic"));
    }

    #[test]
    fn test_project_depth_fallback() {
        // Test depth fallback: meters preferred over feet
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Depth Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("Depth (Meters)".to_string(), crate::decode::DecodedValue {
                    channel_id: "0xC1".to_string(),
                    value: Some(7.3),
                    display_text: Some("7.3".to_string()),
                    layout: None,
                }),
                ("Depth (Feet)".to_string(), crate::decode::DecodedValue {
                    channel_id: "0xC2".to_string(),
                    value: Some(24.1),
                    display_text: Some("24.1".to_string()),
                    layout: None,
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("environment.depth.belowTransducer"));
        assert!((result.get("environment.depth.belowTransducer").unwrap() - 7.3).abs() < 0.001);
    }

    #[test]
    fn test_project_vendor_paths() {
        // Test vendor bandg.* paths
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Wind Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("Upwash".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x85".to_string(),
                    value: Some(5.0),
                    display_text: Some("5.0".to_string()),
                    layout: None,
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("bandg.wind.upwash"));
    }

    #[test]
    fn test_project_unknown_channel() {
        // Test that unmapped but decodable channels go to bandg.unknown.*
        let decoded_frame = crate::decode::DecodedFrame {
            to_address: "Entire System".to_string(),
            from_address: "Normal CPU (Wind Board in H2000)".to_string(),
            command: "Broadcast".to_string(),
            values: [
                ("Linear 5".to_string(), crate::decode::DecodedValue {
                    channel_id: "0x0C".to_string(),
                    value: Some(123.0),
                    display_text: Some("123".to_string()),
                    layout: None,
                }),
            ].into_iter().collect(),
        };

        let result = project(&decoded_frame);
        assert!(result.contains_key("bandg.unknown.0x0C"));
    }
}