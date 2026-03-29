use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// From day of year (1–365, northern hemisphere).
    ///
    /// Days outside 1–365 are clamped. Boundaries follow meteorological convention:
    /// Spring 80–171, Summer 172–264, Autumn 265–355, Winter 356–365 & 1–79.
    #[must_use]
    pub fn from_day(day: u16) -> Self {
        let day = day.clamp(1, 365);
        match day {
            80..=171 => Self::Spring,
            172..=264 => Self::Summer,
            265..=355 => Self::Autumn,
            _ => Self::Winter,
        }
    }

    /// Daylight hours (approximate, mid-latitude ~45°N).
    #[must_use]
    pub fn daylight_hours(&self) -> f32 {
        match self {
            Self::Spring => 12.5,
            Self::Summer => 15.0,
            Self::Autumn => 11.5,
            Self::Winter => 9.0,
        }
    }

    /// Growth modifier (0.0-1.0).
    #[must_use]
    pub fn growth_modifier(&self) -> f32 {
        match self {
            Self::Spring => 0.8,
            Self::Summer => 1.0,
            Self::Autumn => 0.4,
            Self::Winter => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summer_solstice_is_summer() {
        assert_eq!(Season::from_day(172), Season::Summer);
    }
    #[test]
    fn winter_solstice_is_winter() {
        assert_eq!(Season::from_day(356), Season::Winter);
    }
    #[test]
    fn jan_1_is_winter() {
        assert_eq!(Season::from_day(1), Season::Winter);
    }
    #[test]
    fn summer_most_daylight() {
        assert!(Season::Summer.daylight_hours() > Season::Winter.daylight_hours());
    }
    #[test]
    fn winter_no_growth() {
        assert_eq!(Season::Winter.growth_modifier(), 0.0);
    }
    #[test]
    fn summer_max_growth() {
        assert_eq!(Season::Summer.growth_modifier(), 1.0);
    }

    #[test]
    fn day_zero_clamped_to_winter() {
        assert_eq!(Season::from_day(0), Season::Winter);
    }

    #[test]
    fn day_above_365_clamped() {
        assert_eq!(Season::from_day(500), Season::Winter);
    }

    #[test]
    fn season_boundaries() {
        // Last day of each season
        assert_eq!(Season::from_day(79), Season::Winter);
        assert_eq!(Season::from_day(80), Season::Spring);
        assert_eq!(Season::from_day(171), Season::Spring);
        assert_eq!(Season::from_day(172), Season::Summer);
        assert_eq!(Season::from_day(264), Season::Summer);
        assert_eq!(Season::from_day(265), Season::Autumn);
        assert_eq!(Season::from_day(355), Season::Autumn);
        assert_eq!(Season::from_day(356), Season::Winter);
        assert_eq!(Season::from_day(365), Season::Winter);
    }
}
