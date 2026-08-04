use std::collections::HashMap;

/// Extracts sign from layout string for signed numeric values
/// Returns -1 for negative layouts, 1 for positive
fn sign_from_layout(layout: Option<&str>) -> i32 {
    match layout {
        Some("-[data]") | Some("=[data]") | Some("L[data]") | Some("H[data]") => -1,
        _ => 1,
    }
}

/// Formats display text based on layout string
/// Handles special cases like H[data], [data]H, [data]-, etc.
fn display_from_layout(layout: Option<&str>, formatted: &str) -> String {
    match layout {
        None => formatted.to_string(),
        Some("°M") => format!("{}°M", formatted),
        Some("H[data]") => format!("H{}", formatted.trim_start_matches('-')),
        Some("[data]H") => format!("{}H", formatted),
        Some("[data]=") => format!("{}=", formatted),
        Some("[data]-") => format!("{}-", formatted),
        Some("[data]°C") => format!("{}°C", formatted),
        Some("[data]°F") => format!("{}°F", formatted),
        Some("[data]L") => format!("{}L", formatted),
        Some("L[data]") => format!("L{}", formatted),
        Some("[data]z") => format!("{}z", formatted),
        Some("z[data]") => format!("z{}", formatted),
        Some("u[data]") => format!("u{}", formatted),
        Some("d[data]") => format!("d{}", formatted),
        _ => formatted.to_string(),
    }
}

/// Decoded value from a FastNet channel
/// Contains the channel ID, numeric value, display text, and layout information
#[derive(Debug, Clone)]
pub struct DecodedValue {
    pub channel_id: String,
    pub value: Option<f64>,
    pub display_text: Option<String>,
    pub layout: Option<String>,
}

/// Complete decoded FastNet frame
/// Contains frame metadata (addresses, command) and decoded channel values
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub to_address: String,
    pub from_address: String,
    pub command: String,
    pub values: HashMap<String, DecodedValue>,
}

/// Decodes a complete FastNet frame from raw bytes
///
/// # Arguments
/// * `frame` - Raw frame bytes including header and checksum
///
/// # Returns
/// * `Ok(DecodedFrame)` - Successfully decoded frame
/// * `Err(String)` - Error message if decoding fails
///
/// # Frame format
/// - Byte 0: To address
/// - Byte 1: From address  
/// - Byte 2: Body size
/// - Byte 3: Command
/// - Byte 4: Header checksum
/// - Bytes 5..(5+body_size): Body data
/// - Last byte: Body checksum
pub fn decode_frame(frame: &[u8]) -> Result<DecodedFrame, String> {
    if frame.len() < 6 {
        return Err("Frame too short".to_string());
    }

    let to_address = frame[0];
    let from_address = frame[1];
    let body_size = frame[2];
    let command = frame[3];

    let body = &frame[5..frame.len() - 1];

    if body.len() < 2 || body.len() != body_size as usize {
        return Err(format!(
            "Invalid body size: expected={}, actual={}",
            body_size,
            body.len()
        ));
    }

    let address_lookup = crate::constants::address_lookup_reverse();
    let command_lookup = crate::constants::command_lookup_reverse();
    let channel_lookup = crate::constants::channel_lookup_reverse();
    let format_size_map = crate::constants::format_size_map_reverse();

    let to_name = address_lookup
        .get(&to_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", to_address));

    let from_name = address_lookup
        .get(&from_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", from_address));

    let cmd_name = command_lookup
        .get(&command)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", command));

    let mut decoded_data = DecodedFrame {
        to_address: to_name,
        from_address: from_name,
        command: cmd_name,
        values: HashMap::new(),
    };

    // Parse channel records in the body
    // Each channel record: channel_id (1B) + format_byte (1B) + data (variable)
    let mut index = 0;
    while index < body.len() {
        if index + 1 >= body.len() {
            return Err("Insufficient bytes for channel header".to_string());
        }

        let channel_id = body[index];
        let format_byte = body[index + 1];
        let channel_name = channel_lookup
            .get(&channel_id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Unknown (0x{:02X})", channel_id));

        index += 2;

        // Determine data length based on format bits
        let data_length = format_size_map
            .get(&(format_byte & 0x0F))
            .copied()
            .unwrap_or(0);

        if index + data_length > body.len() {
            return Err(format!("Incomplete data for channel 0x{:02X}", channel_id));
        }

        let data_bytes = &body[index..index + data_length];
        index += data_length;

        let decoded_value = decode_format_and_data(channel_id, format_byte, data_bytes);
        decoded_data.values.insert(channel_name, decoded_value);
    }

    Ok(decoded_data)
}

/// Decodes an ASCII frame (typically LatLon position data)
/// ASCII frames contain text data in the body
pub fn decode_ascii_frame(frame: &[u8]) -> Result<DecodedFrame, String> {
    if frame.len() < 6 {
        return Err("Frame too short".to_string());
    }

    let to_address = frame[0];
    let from_address = frame[1];
    let command = frame[3];
    let body = &frame[5..frame.len() - 1];

    if body.is_empty() {
        return Err("Empty body".to_string());
    }

    let channel_id = body[0];
    let data_bytes = &body[2..];

    let command_lookup = crate::constants::command_lookup_reverse();
    let address_lookup = crate::constants::address_lookup_reverse();

    let cmd_name = command_lookup
        .get(&command)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", command));

    let channel_name = cmd_name.clone();

    let ascii_text = match String::from_utf8(data_bytes.to_vec()) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Err("ASCII decode failed".to_string()),
    };

    let to_name = address_lookup
        .get(&to_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", to_address));

    let from_name = address_lookup
        .get(&from_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", from_address));

    let mut decoded_data = DecodedFrame {
        to_address: to_name,
        from_address: from_name,
        command: cmd_name,
        values: HashMap::new(),
    };

    decoded_data.values.insert(
        channel_name,
        DecodedValue {
            channel_id: format!("0x{:02X}", channel_id),
            value: None,
            display_text: Some(ascii_text),
            layout: None,
        },
    );

    Ok(decoded_data)
}

/// Decodes a Light Intensity frame (backlight level)
/// Body contains single byte: 0x00=Off, 0x01=Low, 0x02=Medium, 0x04=High
pub fn decode_light_frame(frame: &[u8]) -> Result<DecodedFrame, String> {
    if frame.len() < 6 {
        return Err("Frame too short".to_string());
    }

    let to_address = frame[0];
    let from_address = frame[1];
    let command = frame[3];
    let body = &frame[5..frame.len() - 1];

    if body.is_empty() {
        return Err("Empty body".to_string());
    }

    let level = body[0];
    let command_lookup = crate::constants::command_lookup_reverse();
    let address_lookup = crate::constants::address_lookup_reverse();

    let cmd_name = command_lookup
        .get(&command)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", command));

    let to_name = address_lookup
        .get(&to_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", to_address));

    let from_name = address_lookup
        .get(&from_address)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown (0x{:02X})", from_address));

    let backlight_levels: HashMap<u8, &str> =
        crate::constants::BACKLIGHT_LEVELS.iter().cloned().collect();
    let display_text = backlight_levels
        .get(&level)
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Unknown ({})", level));

    let mut decoded_data = DecodedFrame {
        to_address: to_name,
        from_address: from_name,
        command: cmd_name,
        values: HashMap::new(),
    };

    decoded_data.values.insert(
        "Backlight".to_string(),
        DecodedValue {
            channel_id: format!("0x{:02X}", 0xC9),
            value: Some(level as f64),
            display_text: Some(display_text),
            layout: None,
        },
    );

    Ok(decoded_data)
}

/// Decodes channel data based on format byte and data bytes
///
/// # Arguments
/// * `channel_id` - The channel identifier
/// * `format_byte` - Format specification byte
/// * `data_bytes` - Raw data bytes for this channel
///
/// # Format byte structure
/// - Bits 7-6: Divisor (0=1, 1=10, 2=100, 3=1000)
/// - Bits 3-0: Format bits (0x01-0x0A, various data formats)
pub fn decode_format_and_data(channel_id: u8, format_byte: u8, data_bytes: &[u8]) -> DecodedValue {
    let divisor_map: HashMap<u8, u32> =
        [(0, 1), (1, 10), (2, 100), (3, 1000)].into_iter().collect();
    let decimal_places_map: HashMap<u32, u8> =
        [(1, 0), (10, 1), (100, 2), (1000, 3)].into_iter().collect();
    let segment_a: HashMap<u8, &'static str> = crate::constants::segment_a_reverse();
    let autopilot_mode_by_low: HashMap<u8, &str> =
        crate::constants::autopilot_mode_by_low_reverse();

    let divisor = divisor_map
        .get(&(format_byte >> 6 & 0b11))
        .copied()
        .unwrap_or(1);
    let decimal_places = decimal_places_map.get(&divisor).copied().unwrap_or(0);
    let format_bits = format_byte & 0b1111;

    if data_bytes.is_empty() {
        return DecodedValue {
            channel_id: format!("0x{:02X}", channel_id),
            value: None,
            display_text: None,
            layout: None,
        };
    }

    let mut layout: Option<String> = None;
    let mut value: Option<f64> = None;
    #[allow(unused_assignments)]
    let mut display_text: Option<String> = None;

    match format_bits {
        // Format 0x01: 16-bit signed integer with divisor
        0x01 => {
            if data_bytes.len() != 2 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let raw = i16::from_be_bytes([data_bytes[0], data_bytes[1]]) as i32;
            if channel_id == 0xB5 {
                // Autopilot mode is a composite value
                let high = (raw >> 8) & 0xFF;
                let low = raw as u8;
                if high == 0x50 {
                    value = Some(raw as f64);
                    display_text = Some("Standby".to_string());
                } else if high == 0x51 || high == 0x59 {
                    value = Some(raw as f64);
                    display_text = autopilot_mode_by_low
                        .get(&low)
                        .map(|s| s.to_string())
                        .or_else(|| Some(format!("Unknown ({})", raw)));
                } else {
                    value = Some(raw as f64);
                    display_text = Some(format!("Unknown ({})", raw));
                }
            } else {
                value = Some(raw as f64 / divisor as f64);
                display_text = Some(format!("{:.1$}", value.unwrap(), decimal_places as usize));
            }
        }
        // Format 0x02: 16-bit unsigned integer with divisor
        0x02 => {
            if data_bytes.len() != 2 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let unsigned = ((data_bytes[0] as u16 & 0b11) << 8) | (data_bytes[1] as u16);
            value = Some(unsigned as f64 / divisor as f64);
            display_text = Some(format!("{:.1$}", value.unwrap(), decimal_places as usize));
        }
        // Format 0x03: 16-bit unsigned with layout symbol
        0x03 => {
            if data_bytes.len() != 2 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            layout = segment_a.get(&data_bytes[0]).map(|s| s.to_string());
            let unsigned = data_bytes[1] as i32;
            let sign = sign_from_layout(layout.as_deref());
            value = Some(sign as f64 * unsigned as f64 / divisor as f64);
            display_text = Some(display_from_layout(
                layout.as_deref(),
                &format!("{:.1$}", value.unwrap(), decimal_places as usize),
            ));
        }
        // Format 0x04: 32-bit unsigned with divisor
        0x04 => {
            if data_bytes.len() != 4 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let unsigned =
                u32::from_be_bytes([data_bytes[0], data_bytes[1], data_bytes[2], data_bytes[3]]);
            value = Some(unsigned as f64 / divisor as f64);
            display_text = Some(format!("{:.1$}", value.unwrap(), decimal_places as usize));
        }
        // Format 0x05: 24-bit time (h:m:s)
        0x05 => {
            if data_bytes.len() != 4 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let h = data_bytes[1] as u32;
            let m = data_bytes[2] as u32;
            let s = data_bytes[3] as u32;
            value = Some(h as f64 * 3600.0 + m as f64 * 60.0 + s as f64);
            display_text = Some(format!("{:02}:{:02}:{:02}", h, m, s));
        }
        // Format 0x06: 4-byte 7-segment display
        0x06 => {
            if data_bytes.len() != 4 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let segment_b: HashMap<u8, &str> =
                crate::constants::SEGMENT_B.iter().cloned().collect();
            display_text = Some(
                data_bytes
                    .iter()
                    .map(|b| segment_b.get(b).copied().unwrap_or("TBC"))
                    .collect::<String>(),
            );
        }
        // Format 0x07: 32-bit with layout symbol
        0x07 => {
            if data_bytes.len() != 4 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            layout = segment_a.get(&data_bytes[1]).map(|s| s.to_string());
            let msb = data_bytes[2] & 0x7F;
            let unsigned = ((msb as u16) << 8) | (data_bytes[3] as u16);
            let sign = sign_from_layout(layout.as_deref());
            value = Some(sign as f64 * unsigned as f64 / divisor as f64);
            display_text = Some(display_from_layout(
                layout.as_deref(),
                &format!("{:.1$}", value.unwrap(), decimal_places as usize),
            ));
        }
        // Format 0x08: 16-bit with layout symbol
        0x08 => {
            if data_bytes.len() != 2 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let segment_code = (data_bytes[0] >> 1) & 0x7F;
            layout = segment_a.get(&segment_code).map(|s| s.to_string());
            let unsigned = (((data_bytes[0] & 0x01) as u16) << 8) | (data_bytes[1] as u16);
            value = Some(unsigned as f64 / divisor as f64);
            display_text = Some(display_from_layout(
                layout.as_deref(),
                &format!("{:.1$}", value.unwrap(), decimal_places as usize),
            ));
        }
        // Format 0x0A: Two 16-bit values (e.g., TWD)
        0x0A => {
            if data_bytes.len() != 4 {
                return DecodedValue {
                    channel_id: format!("0x{:02X}", channel_id),
                    value: None,
                    display_text: None,
                    layout: None,
                };
            }
            let first = i16::from_be_bytes([data_bytes[0], data_bytes[1]]) as f64 / divisor as f64;
            let second = i16::from_be_bytes([data_bytes[2], data_bytes[3]]) as f64 / divisor as f64;
            value = Some(first);
            display_text = Some(format!("{} / {}", first, second));
        }
        _ => {
            return DecodedValue {
                channel_id: format!("0x{:02X}", channel_id),
                value: None,
                display_text: None,
                layout: None,
            };
        }
    }

    DecodedValue {
        channel_id: format!("0x{:02X}", channel_id),
        value,
        display_text,
        layout: layout.map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts a hex string to bytes
    /// Example: "FF050601E7" -> [0xFF, 0x05, 0x06, 0x01, 0xE7]
    fn bytes_from_hex(hex: &str) -> Vec<u8> {
        let hex = hex.replace([' ', '\n', '\t'], "");
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn test_decode_frame_basic() {
        // Test basic frame decoding with Apparent Wind Speed in m/s (channel 0x4F)
        // Frame: FF 05 14 01 E7 8D 81 05 26 3B 31 01 FA 34 47 00 F3 00 CC 9B 47 00 A0 00 09 9B
        // Body size: 0x14 (20 bytes), contains multiple channels
        let frame = bytes_from_hex("FF050601E74F10002600009B");
        let result = decode_frame(&frame);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.to_address, "Entire System");
        assert_eq!(decoded.from_address, "Normal CPU (Wind Board in H2000)");
        assert_eq!(decoded.command, "Broadcast");
    }

    #[test]
    fn test_decode_ascii_frame() {
        // Test LatLon ASCII frame decoding
        // Contains position data: "3352.450S15113.920E"
        let frame = bytes_from_hex("FF050E03034700333335322E3435305331353131332E393230459B");
        let result = decode_ascii_frame(&frame);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.command, "LatLon");
    }

    #[test]
    fn test_decode_light_frame() {
        // Test Light Intensity frame (backlight level)
        // Body contains: 0x04 = High backlight level
        let frame = bytes_from_hex("FF5001C9E704FC");
        let result = decode_light_frame(&frame);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.command, "Light Intensity");
    }

    #[test]
    fn test_decode_apparent_wind_speed() {
        // Test Apparent Wind Speed in m/s (channel 0x4F)
        // Format 0x61: divisor=10, signed 16-bit
        // Data: 0x0050 = 80, value = 80/10 = 8.0 m/s
        let frame = bytes_from_hex("FF050401E74F6100509B");
        let decoded = decode_frame(&frame).unwrap();
        let aws = decoded.values.get("Apparent Wind Speed (m/s)").unwrap();
        assert_eq!(aws.value, Some(8.0));
    }

    #[test]
    fn test_decode_heel_angle() {
        // Test Heel Angle with layout symbol (channel 0x34)
        // Format 0x47: layout=H[data], value=-204 (H20.4 = heel 20.4° port)
        let frame = bytes_from_hex("FF050601E7344700F300CC9B");
        let decoded = decode_frame(&frame).unwrap();
        let heel = decoded.values.get("Heel Angle").unwrap();
        assert_eq!(heel.value, Some(-20.4));
        assert_eq!(heel.layout, Some("H[data]".to_string()));
    }

    #[test]
    fn test_decode_autopilot_mode() {
        // Test Autopilot Mode (channel 0xB5)
        // Composite value: 0x5000 = standby mode (high byte=0x50)
        let frame = bytes_from_hex("FF050401E7B50150009B");
        let decoded = decode_frame(&frame).unwrap();
        let ap_mode = decoded.values.get("Autopilot Mode").unwrap();
        assert_eq!(ap_mode.value, Some(20480.0));
    }
}
