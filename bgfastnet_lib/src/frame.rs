use crate::decode::{decode_frame, decode_ascii_frame, decode_light_frame};
use crate::constants;
use crate::map;

#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub to_address: String,
    pub from_address: String,
    pub command: String,
    pub values: std::collections::HashMap<String, f64>,
}

pub struct FrameBuffer {
    buffer: Vec<u8>,
    max_buffer_size: usize,
    frame_queue: Vec<DecodedFrame>,
    project: bool,
}

impl FrameBuffer {
    pub fn new(max_buffer_size: usize, max_queue_size: usize, project: bool) -> Self {
        FrameBuffer {
            buffer: Vec::with_capacity(max_buffer_size),
            max_buffer_size,
            frame_queue: Vec::with_capacity(max_queue_size),
            project,
        }
    }

    pub fn new_default() -> Self {
        Self::new(8192, 1000, true)
    }

    pub fn add_to_buffer(&mut self, new_data: &[u8]) {
        self.buffer.extend_from_slice(new_data);

        if self.buffer.len() > self.max_buffer_size {
            self.buffer = self.buffer[self.buffer.len() - self.max_buffer_size..].to_vec();
        }
    }

    pub fn get_complete_frames(&mut self) {
        while self.buffer.len() >= 6 {
            let body_size = self.buffer[2];
            let command = self.buffer[3];
            let header_checksum = self.buffer[4];

            let command_lookup = constants::command_lookup_reverse();
            let full_frame_length = 5 + body_size as usize + 1;

            if self.buffer.len() < full_frame_length {
                break;
            }

            let frame = self.buffer[..full_frame_length].to_vec();
            let body = self.buffer[5..full_frame_length - 1].to_vec();
            let body_checksum = self.buffer[full_frame_length - 1];

            if crate::utils::calculate_checksum(&self.buffer[..4]) != header_checksum {
                self.buffer.drain(0..1);
                continue;
            }

            if crate::utils::calculate_checksum(&body) != body_checksum {
                self.buffer.drain(0..1);
                continue;
            }

            self.buffer.drain(0..full_frame_length);

            let command_name = command_lookup
                .get(&command)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Unknown (0x{:02X})", command));

            if constants::IGNORED_COMMANDS.contains(&command) {
                continue;
            }

            self.decode_and_queue_frame(&frame, &command_name);
        }
    }

    fn decode_and_queue_frame(&mut self, frame: &[u8], command_name: &str) {
        let decoded_frame = if command_name == "Broadcast" {
            decode_frame(frame).ok()
        } else if command_name == "LatLon" {
            decode_ascii_frame(frame).ok()
        } else if command_name == "Light Intensity" {
            decode_light_frame(frame).ok()
        } else {
            None
        };

        let decoded_frame = match decoded_frame {
            Some(df) => df,
            None => return,
        };

        if decoded_frame.values.is_empty() {
            return;
        }

        let values = if self.project {
            map::project(&decoded_frame)
        } else {
            decoded_frame.values
                .iter()
                .filter_map(|(k, v)| v.value.map(|val| (k.clone(), val)))
                .collect()
        };

        if values.is_empty() {
            return;
        }

        if self.frame_queue.len() >= self.max_buffer_size {
            self.frame_queue.remove(0);
        }

        self.frame_queue.push(DecodedFrame {
            to_address: decoded_frame.to_address,
            from_address: decoded_frame.from_address,
            command: decoded_frame.command,
            values,
        });
    }

    pub fn frame_queue(&self) -> &[DecodedFrame] {
        &self.frame_queue
    }

    pub fn frame_queue_pop(&mut self) -> Option<DecodedFrame> {
        self.frame_queue.pop()
    }

    pub fn get_buffer_size(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer_contents(&self) -> String {
        self.buffer.iter().map(|b| format!("{:02X}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_basic() {
        let mut fb = FrameBuffer::new_default();
        
        // Test frame: ff 05 14 01 e7 8d 81 05 26 3b 31 01 fa 34 47 00 f3 00 cc 9b 47 00 a0 00 09 9b
        let frame = vec![
            0xFF, 0x05, 0x14, 0x01, 0xE7,
            0x8D, 0x81, 0x05, 0x26, 0x3B, 0x31, 0x01, 0xFA, 0x34, 0x47, 0x00, 0xF3, 0x00, 0xCC, 0x9B, 0x47, 0x00, 0xA0, 0x00, 0x09,
            0x9B,
        ];
        
        fb.add_to_buffer(&frame);
        fb.get_complete_frames();
        
        assert_eq!(fb.frame_queue().len(), 1);
        assert!(fb.frame_queue()[0].values.contains_key("navigation.attitude.roll"));
    }

    #[test]
    fn test_frame_buffer_project_false() {
        let mut fb = FrameBuffer::new(8192, 1000, false);
        
        let frame = vec![
            0xFF, 0x05, 0x14, 0x01, 0xE7,
            0x8D, 0x81, 0x05, 0x26, 0x3B, 0x31, 0x01, 0xFA, 0x34, 0x47, 0x00, 0xF3, 0x00, 0xCC, 0x9B, 0x47, 0x00, 0xA0, 0x00, 0x09,
            0x9B,
        ];
        
        fb.add_to_buffer(&frame);
        fb.get_complete_frames();
        
        assert_eq!(fb.frame_queue().len(), 1);
    }
}
