//! Protocol-agnostic value mapping functions.
//!
//! These functions convert between Daikin-specific enum values and
//! generic numeric representations suitable for various smart home protocols.

use crate::status::{AutoModeWindSpeed, HorizontalDirection, Mode, VerticalDirection, WindSpeed};

/// Mode mapping functions for HVAC operating states.
pub mod mode {
    use super::Mode;

    /// Convert Daikin Mode to current operating state.
    ///
    /// Returns:
    /// - 0: Inactive (Fan mode)
    /// - 1: Idle (Dehumidify mode)
    /// - 2: Heating
    /// - 3: Cooling
    pub fn to_current_state(mode: Option<Mode>) -> u8 {
        match mode {
            Some(Mode::Fan) => 0,
            Some(Mode::Dehumidify) => 1,
            Some(Mode::Heating) => 2,
            Some(Mode::Cooling) => 3,
            // TODO: Auto mode should map based on actual heating/cooling activity
            _ => 0,
        }
    }

    /// Convert Daikin Mode to target operating state.
    ///
    /// Returns:
    /// - Some(0): Auto mode
    /// - Some(1): Heating mode
    /// - Some(2): Cooling mode
    /// - None: Other modes (Fan, Dehumidify)
    pub fn to_target_state(mode: Option<Mode>) -> Option<u8> {
        match mode {
            Some(Mode::Auto) => Some(0),
            Some(Mode::Heating) => Some(1),
            Some(Mode::Cooling) => Some(2),
            _ => None,
        }
    }

    /// Convert target operating state to Daikin Mode.
    ///
    /// - 0 -> Auto
    /// - 1 -> Heating
    /// - 2 -> Cooling
    pub fn from_target_state(state: u8) -> Option<Mode> {
        match state {
            0 => Some(Mode::Auto),
            1 => Some(Mode::Heating),
            2 => Some(Mode::Cooling),
            _ => None,
        }
    }
}

/// Fan speed mapping functions.
///
/// Fan speed is represented as a percentage (0-100%) with Auto mode handled separately.
/// The Daikin device has 6 manual speed levels (Silent, Lev1-5) plus Auto mode.
pub mod fan {
    use super::{AutoModeWindSpeed, WindSpeed};

    /// Fan speed with auto mode as a separate flag.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FanSpeed {
        /// Speed percentage (0-100). Only meaningful when `auto` is false.
        pub percent: u8,
        /// Whether auto mode is enabled.
        pub auto: bool,
    }

    impl FanSpeed {
        /// Create a new fan speed with manual percentage.
        pub fn manual(percent: u8) -> Self {
            Self {
                percent: percent.min(100),
                auto: false,
            }
        }

        /// Create a new fan speed with auto mode enabled.
        pub fn auto() -> Self {
            Self {
                percent: 0,
                auto: true,
            }
        }
    }

    /// Convert WindSpeed enum to FanSpeed (0-100% scale with auto flag).
    ///
    /// Percentage mapping (6 levels evenly distributed):
    /// - Silent: 0%
    /// - Lev1: 20%
    /// - Lev2: 40%
    /// - Lev3: 60%
    /// - Lev4: 80%
    /// - Lev5: 100%
    /// - Auto: auto=true
    pub fn speed_to_fan_speed(speed: Option<WindSpeed>) -> Option<FanSpeed> {
        match speed {
            Some(WindSpeed::Silent) => Some(FanSpeed::manual(0)),
            Some(WindSpeed::Lev1) => Some(FanSpeed::manual(20)),
            Some(WindSpeed::Lev2) => Some(FanSpeed::manual(40)),
            Some(WindSpeed::Lev3) => Some(FanSpeed::manual(60)),
            Some(WindSpeed::Lev4) => Some(FanSpeed::manual(80)),
            Some(WindSpeed::Lev5) => Some(FanSpeed::manual(100)),
            Some(WindSpeed::Auto) => Some(FanSpeed::auto()),
            _ => None,
        }
    }

    /// Convert FanSpeed (0-100% scale with auto flag) to WindSpeed enum.
    ///
    /// Percentage thresholds:
    /// - 0-9%: Silent
    /// - 10-29%: Lev1
    /// - 30-49%: Lev2
    /// - 50-69%: Lev3
    /// - 70-89%: Lev4
    /// - 90-100%: Lev5
    /// - auto=true: Auto
    pub fn fan_speed_to_speed(fan_speed: &FanSpeed) -> WindSpeed {
        if fan_speed.auto {
            return WindSpeed::Auto;
        }
        match fan_speed.percent {
            0..=9 => WindSpeed::Silent,
            10..=29 => WindSpeed::Lev1,
            30..=49 => WindSpeed::Lev2,
            50..=69 => WindSpeed::Lev3,
            70..=89 => WindSpeed::Lev4,
            _ => WindSpeed::Lev5,
        }
    }

    /// Determine AutoModeWindSpeed based on FanSpeed.
    ///
    /// - auto=true or percent >= 50: Auto
    /// - Otherwise: Silent
    pub fn fan_speed_to_auto_mode(fan_speed: &FanSpeed) -> AutoModeWindSpeed {
        if fan_speed.auto || fan_speed.percent >= 50 {
            AutoModeWindSpeed::Auto
        } else {
            AutoModeWindSpeed::Silent
        }
    }
}

/// Swing mode mapping functions for air direction control.
pub mod swing {
    use super::{HorizontalDirection, VerticalDirection};

    /// Check if swing mode is enabled based on vertical direction.
    pub fn to_enabled(direction: Option<VerticalDirection>) -> bool {
        matches!(direction, Some(VerticalDirection::Swing))
    }

    /// Convert swing enabled state to direction settings.
    ///
    /// When swing is disabled, both directions are set to Auto.
    /// When swing is enabled, both directions are set to Swing.
    pub fn from_enabled(enabled: bool) -> (VerticalDirection, HorizontalDirection) {
        if enabled {
            (VerticalDirection::Swing, HorizontalDirection::Swing)
        } else {
            (VerticalDirection::Auto, HorizontalDirection::Auto)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod mode_tests {
        use super::*;

        #[test]
        fn test_to_current_state() {
            assert_eq!(mode::to_current_state(Some(Mode::Fan)), 0);
            assert_eq!(mode::to_current_state(Some(Mode::Dehumidify)), 1);
            assert_eq!(mode::to_current_state(Some(Mode::Heating)), 2);
            assert_eq!(mode::to_current_state(Some(Mode::Cooling)), 3);
            assert_eq!(mode::to_current_state(Some(Mode::Auto)), 0);
            assert_eq!(mode::to_current_state(None), 0);
        }

        #[test]
        fn test_to_target_state() {
            assert_eq!(mode::to_target_state(Some(Mode::Auto)), Some(0));
            assert_eq!(mode::to_target_state(Some(Mode::Heating)), Some(1));
            assert_eq!(mode::to_target_state(Some(Mode::Cooling)), Some(2));
            assert_eq!(mode::to_target_state(Some(Mode::Fan)), None);
            assert_eq!(mode::to_target_state(Some(Mode::Dehumidify)), None);
            assert_eq!(mode::to_target_state(None), None);
        }

        #[test]
        fn test_from_target_state() {
            assert_eq!(mode::from_target_state(0), Some(Mode::Auto));
            assert_eq!(mode::from_target_state(1), Some(Mode::Heating));
            assert_eq!(mode::from_target_state(2), Some(Mode::Cooling));
            assert_eq!(mode::from_target_state(3), None);
            assert_eq!(mode::from_target_state(255), None);
        }

        #[test]
        fn test_roundtrip() {
            for state in 0..=2 {
                let mode = mode::from_target_state(state);
                assert_eq!(mode::to_target_state(mode), Some(state));
            }
        }
    }

    mod fan_tests {
        use super::*;
        use fan::FanSpeed;

        #[test]
        fn test_speed_to_fan_speed() {
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Silent)),
                Some(FanSpeed::manual(0))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Lev1)),
                Some(FanSpeed::manual(20))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Lev2)),
                Some(FanSpeed::manual(40))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Lev3)),
                Some(FanSpeed::manual(60))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Lev4)),
                Some(FanSpeed::manual(80))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Lev5)),
                Some(FanSpeed::manual(100))
            );
            assert_eq!(
                fan::speed_to_fan_speed(Some(WindSpeed::Auto)),
                Some(FanSpeed::auto())
            );
            assert_eq!(fan::speed_to_fan_speed(Some(WindSpeed::Unknown)), None);
            assert_eq!(fan::speed_to_fan_speed(None), None);
        }

        #[test]
        fn test_fan_speed_to_speed() {
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(0)), WindSpeed::Silent);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(9)), WindSpeed::Silent);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(10)), WindSpeed::Lev1);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(29)), WindSpeed::Lev1);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(30)), WindSpeed::Lev2);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(50)), WindSpeed::Lev3);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(70)), WindSpeed::Lev4);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(90)), WindSpeed::Lev5);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::manual(100)), WindSpeed::Lev5);
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::auto()), WindSpeed::Auto);
        }

        #[test]
        fn test_fan_speed_to_auto_mode() {
            assert_eq!(
                fan::fan_speed_to_auto_mode(&FanSpeed::manual(0)),
                AutoModeWindSpeed::Silent
            );
            assert_eq!(
                fan::fan_speed_to_auto_mode(&FanSpeed::manual(49)),
                AutoModeWindSpeed::Silent
            );
            assert_eq!(
                fan::fan_speed_to_auto_mode(&FanSpeed::manual(50)),
                AutoModeWindSpeed::Auto
            );
            assert_eq!(
                fan::fan_speed_to_auto_mode(&FanSpeed::manual(100)),
                AutoModeWindSpeed::Auto
            );
            assert_eq!(
                fan::fan_speed_to_auto_mode(&FanSpeed::auto()),
                AutoModeWindSpeed::Auto
            );
        }

        #[test]
        fn test_roundtrip_fan_speed() {
            // Manual speeds roundtrip (using center of each range)
            let test_cases = [
                (0, WindSpeed::Silent),
                (20, WindSpeed::Lev1),
                (40, WindSpeed::Lev2),
                (60, WindSpeed::Lev3),
                (80, WindSpeed::Lev4),
                (100, WindSpeed::Lev5),
            ];
            for (percent, expected_speed) in test_cases {
                let fan_speed = FanSpeed::manual(percent);
                assert_eq!(fan::fan_speed_to_speed(&fan_speed), expected_speed);
            }

            // Auto roundtrip
            assert_eq!(fan::fan_speed_to_speed(&FanSpeed::auto()), WindSpeed::Auto);
        }
    }

    mod swing_tests {
        use super::*;

        #[test]
        fn test_to_enabled() {
            assert!(swing::to_enabled(Some(VerticalDirection::Swing)));
            assert!(!swing::to_enabled(Some(VerticalDirection::Auto)));
            assert!(!swing::to_enabled(Some(VerticalDirection::Center)));
            assert!(!swing::to_enabled(None));
        }

        #[test]
        fn test_from_enabled() {
            assert_eq!(
                swing::from_enabled(true),
                (VerticalDirection::Swing, HorizontalDirection::Swing)
            );
            assert_eq!(
                swing::from_enabled(false),
                (VerticalDirection::Auto, HorizontalDirection::Auto)
            );
        }
    }
}
