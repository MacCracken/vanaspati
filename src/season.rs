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

    /// Growth modifier (0.0–1.0).
    #[must_use]
    pub fn growth_modifier(&self) -> f32 {
        match self {
            Self::Spring => 0.8,
            Self::Summer => 1.0,
            Self::Autumn => 0.4,
            Self::Winter => 0.0,
        }
    }

    /// Season from day of year and latitude (hemisphere-aware).
    ///
    /// For latitudes >= 0 (northern hemisphere), uses the standard mapping.
    /// For latitudes < 0 (southern hemisphere), shifts by 182 days (inverts seasons).
    ///
    /// - `day` — day of year (1–365)
    /// - `latitude_deg` — latitude (degrees, negative for southern hemisphere)
    #[must_use]
    pub fn from_day_latitude(day: u16, latitude_deg: f32) -> Self {
        if latitude_deg >= 0.0 {
            Self::from_day(day)
        } else {
            let day = day.clamp(1, 365);
            let shifted = ((day as u32 - 1 + 182) % 365 + 1) as u16;
            Self::from_day(shifted)
        }
    }
}

/// Daylight hours at a given day and latitude using the sunrise equation.
///
/// Solar declination: `δ = 23.44 × sin(2π × (284 + day) / 365)` degrees
///
/// Hour angle: `ω = arccos(-tan(φ) × tan(δ))`
///
/// Daylight: `D = (2/15) × ω` hours
///
/// Returns 0.0 for polar night, 24.0 for midnight sun.
///
/// - `day_of_year` — day (1–365, clamped)
/// - `latitude_deg` — latitude (degrees, negative for southern hemisphere)
#[must_use]
pub fn daylight_hours_at(day_of_year: u16, latitude_deg: f32) -> f32 {
    let day = day_of_year.clamp(1, 365) as f32;
    // Clamp latitude to avoid tan(±90°) singularity
    let lat_rad = latitude_deg.clamp(-89.99, 89.99).to_radians();

    // Solar declination (radians)
    let declination =
        (23.44_f32).to_radians() * (2.0 * std::f32::consts::PI * (284.0 + day) / 365.0).sin();

    let cos_hour_angle = -(lat_rad.tan()) * declination.tan();

    // Polar night / midnight sun
    if cos_hour_angle >= 1.0 {
        tracing::trace!(
            day_of_year,
            latitude_deg,
            hours = 0.0,
            "daylight_hours_at (polar night)"
        );
        return 0.0;
    }
    if cos_hour_angle <= -1.0 {
        tracing::trace!(
            day_of_year,
            latitude_deg,
            hours = 24.0,
            "daylight_hours_at (midnight sun)"
        );
        return 24.0;
    }

    let hour_angle_deg = cos_hour_angle.acos().to_degrees();
    let hours = (2.0 / 15.0) * hour_angle_deg;
    tracing::trace!(
        day_of_year,
        latitude_deg,
        declination_deg = declination.to_degrees(),
        hours,
        "daylight_hours_at"
    );
    hours
}

/// Continuous growth modifier based on daylight hours at a given day and latitude.
///
/// Maps daylight linearly: 8 hours → 0.0, 16 hours → 1.0, clamped to `[0.0, 1.0]`.
///
/// `modifier = ((daylight - 8) / 8).clamp(0.0, 1.0)`
///
/// - `day_of_year` — day (1–365)
/// - `latitude_deg` — latitude (degrees)
#[must_use]
pub fn growth_modifier_at(day_of_year: u16, latitude_deg: f32) -> f32 {
    let hours = daylight_hours_at(day_of_year, latitude_deg);
    let modifier = ((hours - 8.0) / 8.0).clamp(0.0, 1.0);
    tracing::trace!(
        day_of_year,
        latitude_deg,
        hours,
        modifier,
        "growth_modifier_at"
    );
    modifier
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

    // --- Latitude-parameterized tests ---

    #[test]
    fn daylight_summer_solstice_45n() {
        let hours = daylight_hours_at(172, 45.0);
        assert!(
            (hours - 15.4).abs() < 0.5,
            "45°N summer solstice should be ~15.4h, got {hours}"
        );
    }

    #[test]
    fn daylight_winter_solstice_45n() {
        let hours = daylight_hours_at(356, 45.0);
        assert!(
            (hours - 8.6).abs() < 0.5,
            "45°N winter solstice should be ~8.6h, got {hours}"
        );
    }

    #[test]
    fn daylight_equinox_near_12() {
        let hours = daylight_hours_at(80, 45.0); // vernal equinox
        assert!(
            (hours - 12.0).abs() < 0.5,
            "equinox should be ~12h, got {hours}"
        );
    }

    #[test]
    fn daylight_equator_always_near_12() {
        for day in [1, 80, 172, 265, 356] {
            let hours = daylight_hours_at(day, 0.0);
            assert!(
                (hours - 12.0).abs() < 0.3,
                "equator day {day} should be ~12h, got {hours}"
            );
        }
    }

    #[test]
    fn daylight_polar_midnight_sun() {
        let hours = daylight_hours_at(172, 70.0); // 70°N, summer solstice
        assert_eq!(hours, 24.0, "should be midnight sun");
    }

    #[test]
    fn daylight_north_pole_summer() {
        assert_eq!(
            daylight_hours_at(172, 90.0),
            24.0,
            "north pole summer = 24h"
        );
    }

    #[test]
    fn daylight_north_pole_winter() {
        assert_eq!(daylight_hours_at(356, 90.0), 0.0, "north pole winter = 0h");
    }

    #[test]
    fn daylight_south_pole_summer() {
        assert_eq!(
            daylight_hours_at(356, -90.0),
            24.0,
            "south pole summer = 24h"
        );
    }

    #[test]
    fn daylight_polar_night() {
        let hours = daylight_hours_at(356, 70.0); // 70°N, winter solstice
        assert_eq!(hours, 0.0, "should be polar night");
    }

    #[test]
    fn daylight_day_zero_clamped() {
        let hours = daylight_hours_at(0, 45.0);
        assert!(hours > 0.0); // clamped to day 1
    }

    #[test]
    fn southern_hemisphere_inverted() {
        // January (day 15) in southern hemisphere should be summer
        let south = Season::from_day_latitude(15, -35.0);
        assert_eq!(south, Season::Summer, "Jan in southern hemisphere = summer");
    }

    #[test]
    fn from_day_latitude_north_matches_from_day() {
        for day in [1, 80, 172, 265, 356] {
            assert_eq!(
                Season::from_day_latitude(day, 45.0),
                Season::from_day(day),
                "northern hemisphere should match standard from_day"
            );
        }
    }

    #[test]
    fn growth_modifier_at_summer_high() {
        let m = growth_modifier_at(172, 45.0); // ~15.4h daylight
        assert!(m > 0.8, "summer should have high growth modifier, got {m}");
    }

    #[test]
    fn growth_modifier_at_winter_low() {
        let m = growth_modifier_at(356, 45.0); // ~8.6h daylight
        assert!(m < 0.15, "winter should have low growth modifier, got {m}");
    }

    #[test]
    fn growth_modifier_at_equator_stable() {
        let summer = growth_modifier_at(172, 0.0);
        let winter = growth_modifier_at(356, 0.0);
        assert!(
            (summer - winter).abs() < 0.1,
            "equator should have stable growth year-round"
        );
    }

    #[test]
    fn growth_modifier_clamps_zero_one() {
        for day in 1..=365 {
            for lat in [-70.0, -45.0, 0.0, 45.0, 70.0] {
                let m = growth_modifier_at(day, lat);
                assert!(
                    (0.0..=1.0).contains(&m),
                    "modifier must be 0-1, got {m} at day={day} lat={lat}"
                );
            }
        }
    }
}
