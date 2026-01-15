//! HomeKit-specific mode mapping functions.
//!
//! This module provides mapping between Daikin's Mode enum and HomeKit's
//! HeaterCooler characteristic states.

use dsiot::status::Mode;

/// Convert Daikin Mode to HomeKit CurrentHeaterCoolerState.
///
/// HomeKit HAP spec values:
/// - 0: Inactive
/// - 1: Idle
/// - 2: Heating
/// - 3: Cooling
pub fn to_current_state(mode: Option<Mode>) -> u8 {
    match mode {
        Some(Mode::Fan) => 0,        // Inactive
        Some(Mode::Dehumidify) => 1, // Idle
        Some(Mode::Heating) => 2,
        Some(Mode::Cooling) => 3,
        // Auto mode: ideally should check actual heating/cooling activity
        // For now, report as Inactive since we can't determine actual state
        _ => 0,
    }
}

/// Convert Daikin Mode to HomeKit TargetHeaterCoolerState.
///
/// HomeKit HAP spec values:
/// - 0: Auto
/// - 1: Heat
/// - 2: Cool
///
/// Returns None for modes not supported by HomeKit (Fan, Dehumidify).
pub fn to_target_state(mode: Option<Mode>) -> Option<u8> {
    match mode {
        Some(Mode::Auto) => Some(0),
        Some(Mode::Heating) => Some(1),
        Some(Mode::Cooling) => Some(2),
        _ => None,
    }
}

/// Convert HomeKit TargetHeaterCoolerState to Daikin Mode.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_current_state() {
        assert_eq!(to_current_state(Some(Mode::Fan)), 0);
        assert_eq!(to_current_state(Some(Mode::Dehumidify)), 1);
        assert_eq!(to_current_state(Some(Mode::Heating)), 2);
        assert_eq!(to_current_state(Some(Mode::Cooling)), 3);
        assert_eq!(to_current_state(Some(Mode::Auto)), 0);
        assert_eq!(to_current_state(None), 0);
    }

    #[test]
    fn test_to_target_state() {
        assert_eq!(to_target_state(Some(Mode::Auto)), Some(0));
        assert_eq!(to_target_state(Some(Mode::Heating)), Some(1));
        assert_eq!(to_target_state(Some(Mode::Cooling)), Some(2));
        assert_eq!(to_target_state(Some(Mode::Fan)), None);
        assert_eq!(to_target_state(Some(Mode::Dehumidify)), None);
        assert_eq!(to_target_state(None), None);
    }

    #[test]
    fn test_from_target_state() {
        assert_eq!(from_target_state(0), Some(Mode::Auto));
        assert_eq!(from_target_state(1), Some(Mode::Heating));
        assert_eq!(from_target_state(2), Some(Mode::Cooling));
        assert_eq!(from_target_state(3), None);
        assert_eq!(from_target_state(255), None);
    }

    #[test]
    fn test_roundtrip() {
        for state in 0..=2 {
            let mode = from_target_state(state);
            assert_eq!(to_target_state(mode), Some(state));
        }
    }
}
