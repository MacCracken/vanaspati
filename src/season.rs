use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Season { Spring, Summer, Autumn, Winter }

impl Season {
    /// From day of year (1-365, northern hemisphere).
    #[must_use]
    pub fn from_day(day: u16) -> Self {
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
        match self { Self::Spring => 12.5, Self::Summer => 15.0, Self::Autumn => 11.5, Self::Winter => 9.0 }
    }

    /// Growth modifier (0.0-1.0).
    #[must_use]
    pub fn growth_modifier(&self) -> f32 {
        match self { Self::Spring => 0.8, Self::Summer => 1.0, Self::Autumn => 0.4, Self::Winter => 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summer_solstice_is_summer() { assert_eq!(Season::from_day(172), Season::Summer); }
    #[test]
    fn winter_solstice_is_winter() { assert_eq!(Season::from_day(355), Season::Autumn); }
    #[test]
    fn jan_1_is_winter() { assert_eq!(Season::from_day(1), Season::Winter); }
    #[test]
    fn summer_most_daylight() {
        assert!(Season::Summer.daylight_hours() > Season::Winter.daylight_hours());
    }
    #[test]
    fn winter_no_growth() { assert_eq!(Season::Winter.growth_modifier(), 0.0); }
    #[test]
    fn summer_max_growth() { assert_eq!(Season::Summer.growth_modifier(), 1.0); }
}
