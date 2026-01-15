//! HomeKit-specific fan speed mapping functions.
//!
//! This module provides mapping between Daikin's WindSpeed enum and HomeKit's
//! rotation speed characteristic (1.0-7.0 scale).

use dsiot::ValueConstraints;
use dsiot::status::{AutoModeWindSpeed, WindSpeed};

/// Constraints for HomeKit fan speed scale (0-7, step 1).
pub fn fan_speed_constraints() -> ValueConstraints {
    ValueConstraints::new(0.0, 7.0, 1.0)
}

/// Convert WindSpeed enum to HomeKit's 1.0-7.0 scale.
///
/// Scale mapping:
/// - 1.0: Silent
/// - 2.0: Level 1
/// - 3.0: Level 2
/// - 4.0: Level 3
/// - 5.0: Level 4
/// - 6.0: Level 5
/// - 7.0: Auto
pub fn speed_to_scale(speed: Option<WindSpeed>) -> Option<f32> {
    match speed {
        Some(WindSpeed::Silent) => Some(1.0),
        Some(WindSpeed::Lev1) => Some(2.0),
        Some(WindSpeed::Lev2) => Some(3.0),
        Some(WindSpeed::Lev3) => Some(4.0),
        Some(WindSpeed::Lev4) => Some(5.0),
        Some(WindSpeed::Lev5) => Some(6.0),
        Some(WindSpeed::Auto) => Some(7.0),
        _ => None,
    }
}

/// Convert HomeKit's 1.0-7.0 scale to WindSpeed enum.
pub fn scale_to_speed(scale: f32) -> WindSpeed {
    match scale as u8 {
        1 => WindSpeed::Silent,
        2 => WindSpeed::Lev1,
        3 => WindSpeed::Lev2,
        4 => WindSpeed::Lev3,
        5 => WindSpeed::Lev4,
        6 => WindSpeed::Lev5,
        _ => WindSpeed::Auto,
    }
}

/// Determine AutoModeWindSpeed based on HomeKit scale value.
///
/// Uses 50.0 as threshold (for percentage-based input):
/// - Below 50.0: Silent
/// - 50.0 and above: Auto
pub fn scale_to_auto_mode(scale: f32) -> AutoModeWindSpeed {
    if scale < 50.0 {
        AutoModeWindSpeed::Silent
    } else {
        AutoModeWindSpeed::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_to_scale() {
        assert_eq!(speed_to_scale(Some(WindSpeed::Silent)), Some(1.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Lev1)), Some(2.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Lev2)), Some(3.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Lev3)), Some(4.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Lev4)), Some(5.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Lev5)), Some(6.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Auto)), Some(7.0));
        assert_eq!(speed_to_scale(Some(WindSpeed::Unknown)), None);
        assert_eq!(speed_to_scale(None), None);
    }

    #[test]
    fn test_scale_to_speed() {
        assert_eq!(scale_to_speed(1.0), WindSpeed::Silent);
        assert_eq!(scale_to_speed(2.0), WindSpeed::Lev1);
        assert_eq!(scale_to_speed(3.0), WindSpeed::Lev2);
        assert_eq!(scale_to_speed(4.0), WindSpeed::Lev3);
        assert_eq!(scale_to_speed(5.0), WindSpeed::Lev4);
        assert_eq!(scale_to_speed(6.0), WindSpeed::Lev5);
        assert_eq!(scale_to_speed(7.0), WindSpeed::Auto);
        assert_eq!(scale_to_speed(0.0), WindSpeed::Auto);
        assert_eq!(scale_to_speed(100.0), WindSpeed::Auto);
    }

    #[test]
    fn test_scale_to_auto_mode() {
        assert_eq!(scale_to_auto_mode(0.0), AutoModeWindSpeed::Silent);
        assert_eq!(scale_to_auto_mode(49.9), AutoModeWindSpeed::Silent);
        assert_eq!(scale_to_auto_mode(50.0), AutoModeWindSpeed::Auto);
        assert_eq!(scale_to_auto_mode(100.0), AutoModeWindSpeed::Auto);
    }

    #[test]
    fn test_roundtrip() {
        let speeds = [
            WindSpeed::Silent,
            WindSpeed::Lev1,
            WindSpeed::Lev2,
            WindSpeed::Lev3,
            WindSpeed::Lev4,
            WindSpeed::Lev5,
            WindSpeed::Auto,
        ];
        for speed in speeds {
            let scale = speed_to_scale(Some(speed)).unwrap();
            assert_eq!(scale_to_speed(scale), speed);
        }
    }
}
