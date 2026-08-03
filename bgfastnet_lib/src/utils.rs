/// Calculates the FastNet protocol checksum
/// 
/// The checksum is calculated as the two's complement of the sum of all bytes.
/// Formula: checksum = (!sum + 1) & 0xFF
/// 
/// # Arguments
/// * `data` - The data bytes to calculate checksum for
/// 
/// # Returns
/// The calculated checksum as a u8
pub fn calculate_checksum(data: &[u8]) -> u8 {
    let sum: u16 = data.iter().map(|&b| b as u16).sum();
    (!sum).wrapping_add(1) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_empty() {
        assert_eq!(calculate_checksum(&[]), 0);
    }

    #[test]
    fn test_checksum_single_byte() {
        assert_eq!(calculate_checksum(&[0x00]), 0x00);
        assert_eq!(calculate_checksum(&[0xFF]), 0x01);
    }

    #[test]
    fn test_checksum_known_values() {
        // Frame: ff 05 14 01 e7 8d 81 05 26 3b 31 01 fa 34 47 00 f3 00 cc 9b 47 00 a0 00 09 9b
        // Header: ff 05 14 01, checksum: e7
        let header = [0xFF, 0x05, 0x14, 0x01];
        assert_eq!(calculate_checksum(&header), 0xE7);

        // Body: 8d 81 05 26 3b 31 01 fa 34 47 00 f3 00 cc 9b 47 00 a0 00 09
        // checksum: 9b
        let body = [
            0x8D, 0x81, 0x05, 0x26, 0x3B, 0x31, 0x01, 0xFA, 0x34, 0x47, 0x00, 0xF3, 0x00,
            0xCC, 0x9B, 0x47, 0x00, 0xA0, 0x00, 0x09,
        ];
        assert_eq!(calculate_checksum(&body), 0x9B);
    }
}