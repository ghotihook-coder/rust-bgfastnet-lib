use std::collections::HashMap;

/// Master lookup: Signal K path -> NMEA2000 PGN
/// This maps Signal K paths (from bgfastnet_lib) to NMEA2000 PGNs.
/// Only Signal K paths with NMEA2000 equivalents are included.
pub static SIGNALK_TO_PGN: &[(&str, u32, &str)] = &[
    // Attitude
    ("navigation.attitude.roll", 127240, "Attitude"),
    ("navigation.attitude.pitch", 127240, "Attitude"),
    // Rate of Turn
    ("navigation.rateOfTurn", 127245, "Rate of Turn"),
    // Leeway
    ("navigation.leewayAngle", 127249, "Leeway"),
    // Heading
    ("navigation.headingTrue", 127250, "Heading"),
    ("navigation.headingMagnetic", 127250, "Heading"),
    // Vessel Heading (includes rudder angle)
    ("steering.rudderAngle", 127251, "Vessel Heading"),
    // Speed
    ("navigation.speedThroughWater", 127257, "Speed (Boat Speed)"),
    // Wind Speed
    ("environment.wind.speedApparent", 128258, "Apparent Wind Speed"),
    ("environment.wind.speedTrue", 128259, "True Wind Speed"),
    // Water Depth
    ("environment.depth.belowTransducer", 128267, "Water Depth"),
    // Distance
    ("navigation.log", 128275, "Distance"),
    // Position
    ("navigation.position", 129025, "Position (Latitude)"),
    // COG & SOG
    ("navigation.speedOverGround", 129291, "COG & SOG"),
    ("navigation.courseOverGroundTrue", 129291, "COG & SOG"),
];

/// Get NMEA2000 PGN for a Signal K path
pub fn get_pgn_for_path(path: &str) -> Option<(u32, &str)> {
    SIGNALK_TO_PGN
        .iter()
        .find(|(p, _, _)| *p == path)
        .map(|(_, pgn, name)| (*pgn, *name))
}

/// Get all Signal K paths that map to a specific PGN
pub fn get_paths_for_pgn(pgn: u32) -> Vec<&'static str> {
    SIGNALK_TO_PGN
        .iter()
        .filter(|(_, p, _)| *p == pgn)
        .map(|(p, _, _)| *p)
        .collect()
}

/// Get PGN name
pub fn get_pgn_name(pgn: u32) -> Option<&'static str> {
    SIGNALK_TO_PGN
        .iter()
        .find(|(_, p, _)| *p == pgn)
        .map(|(_, _, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pgn_for_path() {
        assert_eq!(get_pgn_for_path("environment.wind.speedApparent"), Some((128258, "Apparent Wind Speed")));
        assert_eq!(get_pgn_for_path("navigation.headingTrue"), Some((127250, "Heading")));
        assert_eq!(get_pgn_for_path("navigation.position"), Some((129025, "Position (Latitude)")));
        assert_eq!(get_pgn_for_path("unknown.path"), None);
    }

    #[test]
    fn test_get_paths_for_pgn() {
        let paths = get_paths_for_pgn(127250);
        assert!(paths.contains(&"navigation.headingTrue"));
        assert!(paths.contains(&"navigation.headingMagnetic"));
    }

    #[test]
    fn test_get_pgn_name() {
        assert_eq!(get_pgn_name(128258), Some("Apparent Wind Speed"));
        assert_eq!(get_pgn_name(99999), None);
    }
}