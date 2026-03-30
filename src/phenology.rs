use serde::{Deserialize, Serialize};

/// Phenological event in the plant lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PhenologicalEvent {
    /// Dormancy broken — chilling requirement met, buds swell.
    DormancyBreak,
    /// Bud break — first visible leaf emergence.
    BudBreak,
    /// Full leaf out — canopy fully expanded.
    LeafOut,
    /// Flowering — reproductive structures open.
    Flowering,
    /// Fruit set — pollinated flowers develop fruit.
    FruitSet,
    /// Leaf senescence — leaves begin to change color and drop.
    LeafSenescence,
    /// Dormancy onset — growth ceases for winter.
    DormancyOnset,
}

/// Growing degree day (GDD) contribution for a single day.
///
/// `GDD = max(0, T_mean - T_base)`
///
/// Accumulates heat units above a base temperature. Plants require
/// a species-specific sum of GDD to reach each phenological stage.
///
/// - `daily_mean_temp_c` — daily mean temperature (°C)
/// - `base_temp_c` — base temperature below which no development occurs (°C)
#[must_use]
#[inline]
pub fn growing_degree_days(daily_mean_temp_c: f32, base_temp_c: f32) -> f32 {
    let gdd = (daily_mean_temp_c - base_temp_c).max(0.0);
    tracing::trace!(daily_mean_temp_c, base_temp_c, gdd, "growing_degree_days");
    gdd
}

/// Accumulated GDD over a sequence of daily mean temperatures.
///
/// `GDD_total = Σ max(0, T_i - T_base)`
///
/// - `daily_temps_c` — slice of daily mean temperatures (°C)
/// - `base_temp_c` — base temperature (°C)
#[must_use]
pub fn accumulated_gdd(daily_temps_c: &[f32], base_temp_c: f32) -> f32 {
    let total: f32 = daily_temps_c
        .iter()
        .map(|&t| (t - base_temp_c).max(0.0))
        .sum();
    tracing::trace!(
        days = daily_temps_c.len(),
        base_temp_c,
        total,
        "accumulated_gdd"
    );
    total
}

/// GDD threshold for a phenological event (°C·days).
///
/// Typical values for temperate deciduous trees (base temperature 5°C):
/// - DormancyBreak: 0 (triggered by chilling, not GDD)
/// - BudBreak: 150 °C·days
/// - LeafOut: 300 °C·days
/// - Flowering: 500 °C·days
/// - FruitSet: 800 °C·days
/// - LeafSenescence: 1800 °C·days (approximate, also photoperiod-driven)
/// - DormancyOnset: 0 (triggered by photoperiod/frost, not GDD)
///
/// Returns 0.0 for events not primarily GDD-driven.
#[must_use]
pub fn gdd_threshold(event: PhenologicalEvent) -> f32 {
    let threshold = match event {
        PhenologicalEvent::DormancyBreak => 0.0,
        PhenologicalEvent::BudBreak => 150.0,
        PhenologicalEvent::LeafOut => 300.0,
        PhenologicalEvent::Flowering => 500.0,
        PhenologicalEvent::FruitSet => 800.0,
        PhenologicalEvent::LeafSenescence => 1800.0,
        PhenologicalEvent::DormancyOnset => 0.0,
    };
    tracing::trace!(?event, threshold, "gdd_threshold");
    threshold
}

/// Check if a GDD-driven phenological event has been reached.
///
/// - `accumulated_gdd` — accumulated growing degree days since dormancy break (°C·days)
/// - `event` — phenological event to check
#[must_use]
#[inline]
pub fn event_reached(accumulated_gdd: f32, event: PhenologicalEvent) -> bool {
    let threshold = gdd_threshold(event);
    if threshold <= 0.0 {
        return false; // not GDD-driven
    }
    accumulated_gdd >= threshold
}

/// Chilling hour contribution for a single hour (0.0 or 1.0).
///
/// The chilling model counts hours where temperature is in the effective
/// range of 0–7.2°C (Utah model simplified). Temperatures outside this
/// range do not contribute to breaking dormancy.
///
/// - `temp_celsius` — hourly temperature (°C)
#[must_use]
#[inline]
pub fn chilling_contribution(temp_celsius: f32) -> f32 {
    if (0.0..=7.2).contains(&temp_celsius) {
        1.0
    } else {
        0.0
    }
}

/// Accumulated chilling hours from a sequence of hourly temperatures.
///
/// Counts hours in the 0–7.2°C range (Utah model simplified).
/// Deciduous trees typically require 400–1500 chilling hours to break dormancy.
///
/// - `hourly_temps_c` — slice of hourly temperatures (°C)
#[must_use]
pub fn accumulated_chill(hourly_temps_c: &[f32]) -> f32 {
    let total: f32 = hourly_temps_c
        .iter()
        .map(|&t| chilling_contribution(t))
        .sum();
    tracing::trace!(hours = hourly_temps_c.len(), total, "accumulated_chill");
    total
}

/// Check if chilling requirement is met for dormancy break.
///
/// Typical requirements:
/// - Low chill (almond, peach): 200–400 hours
/// - Moderate (apple, cherry): 500–1000 hours
/// - High chill (oak, beech): 1000–1500 hours
///
/// - `accumulated_chill_hours` — total chilling hours accumulated
/// - `requirement_hours` — species-specific chilling requirement (hours)
#[must_use]
#[inline]
pub fn dormancy_broken(accumulated_chill_hours: f32, requirement_hours: f32) -> bool {
    if requirement_hours <= 0.0 {
        return true;
    }
    accumulated_chill_hours >= requirement_hours
}

/// Photoperiod-triggered leaf senescence check.
///
/// Leaf senescence in temperate deciduous trees is primarily triggered
/// by shortening days (photoperiod) combined with cooling temperatures.
///
/// Returns `true` when daylight drops below threshold AND temperature
/// drops below the cooling threshold.
///
/// - `daylight_hours` — current day length (hours)
/// - `daylight_threshold` — photoperiod trigger (hours, typically 12.0)
/// - `temp_celsius` — current temperature (°C)
/// - `temp_threshold_c` — temperature trigger (°C, typically 10.0)
#[must_use]
#[inline]
pub fn senescence_triggered(
    daylight_hours: f32,
    daylight_threshold: f32,
    temp_celsius: f32,
    temp_threshold_c: f32,
) -> bool {
    let triggered = daylight_hours < daylight_threshold && temp_celsius < temp_threshold_c;
    tracing::trace!(
        daylight_hours,
        daylight_threshold,
        temp_celsius,
        temp_threshold_c,
        triggered,
        "senescence_triggered"
    );
    triggered
}

/// Photoperiod-triggered dormancy onset check.
///
/// Dormancy is triggered by very short days OR first hard frost.
///
/// - `daylight_hours` — current day length (hours)
/// - `daylight_threshold` — photoperiod trigger (hours, typically 10.0)
/// - `temp_celsius` — current temperature (°C)
/// - `frost_threshold_c` — hard frost temperature (°C, typically -2.0)
#[must_use]
#[inline]
pub fn dormancy_onset_triggered(
    daylight_hours: f32,
    daylight_threshold: f32,
    temp_celsius: f32,
    frost_threshold_c: f32,
) -> bool {
    let triggered = daylight_hours < daylight_threshold || temp_celsius < frost_threshold_c;
    tracing::trace!(
        daylight_hours,
        daylight_threshold,
        temp_celsius,
        frost_threshold_c,
        triggered,
        "dormancy_onset_triggered"
    );
    triggered
}

/// Progress through GDD-driven phenological stages (0.0–1.0).
///
/// Returns the fraction of development toward the next GDD-driven event.
/// Useful for smooth visual transitions (e.g., gradual leaf emergence).
///
/// `progress = (accumulated / threshold).clamp(0.0, 1.0)`
///
/// - `accumulated_gdd` — accumulated growing degree days (°C·days)
/// - `event` — target phenological event
#[must_use]
#[inline]
pub fn phenological_progress(accumulated_gdd: f32, event: PhenologicalEvent) -> f32 {
    let threshold = gdd_threshold(event);
    if threshold <= 0.0 {
        return 0.0;
    }
    let progress = (accumulated_gdd / threshold).clamp(0.0, 1.0);
    tracing::trace!(
        accumulated_gdd,
        ?event,
        threshold,
        progress,
        "phenological_progress"
    );
    progress
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- GDD ---

    #[test]
    fn gdd_warm_day() {
        let g = growing_degree_days(20.0, 5.0);
        assert!((g - 15.0).abs() < 0.01);
    }

    #[test]
    fn gdd_cold_day() {
        assert_eq!(growing_degree_days(3.0, 5.0), 0.0);
    }

    #[test]
    fn gdd_at_base_temp() {
        assert_eq!(growing_degree_days(5.0, 5.0), 0.0);
    }

    #[test]
    fn accumulated_gdd_basic() {
        let temps = [10.0, 15.0, 20.0, 5.0, 3.0]; // base=5
        let total = accumulated_gdd(&temps, 5.0);
        // 5 + 10 + 15 + 0 + 0 = 30
        assert!((total - 30.0).abs() < 0.01, "got {total}");
    }

    #[test]
    fn accumulated_gdd_empty() {
        assert_eq!(accumulated_gdd(&[], 5.0), 0.0);
    }

    #[test]
    fn accumulated_gdd_all_cold() {
        let temps = [0.0, -5.0, 2.0, 4.0];
        assert_eq!(accumulated_gdd(&temps, 5.0), 0.0);
    }

    // --- Thresholds ---

    #[test]
    fn thresholds_increase_through_season() {
        let bud = gdd_threshold(PhenologicalEvent::BudBreak);
        let leaf = gdd_threshold(PhenologicalEvent::LeafOut);
        let flower = gdd_threshold(PhenologicalEvent::Flowering);
        let fruit = gdd_threshold(PhenologicalEvent::FruitSet);
        assert!(bud < leaf);
        assert!(leaf < flower);
        assert!(flower < fruit);
    }

    #[test]
    fn dormancy_events_not_gdd_driven() {
        assert_eq!(gdd_threshold(PhenologicalEvent::DormancyBreak), 0.0);
        assert_eq!(gdd_threshold(PhenologicalEvent::DormancyOnset), 0.0);
    }

    // --- Event reached ---

    #[test]
    fn event_reached_bud_break() {
        assert!(event_reached(200.0, PhenologicalEvent::BudBreak));
        assert!(!event_reached(100.0, PhenologicalEvent::BudBreak));
    }

    #[test]
    fn event_reached_dormancy_always_false() {
        // Dormancy events are not GDD-driven
        assert!(!event_reached(10000.0, PhenologicalEvent::DormancyBreak));
        assert!(!event_reached(10000.0, PhenologicalEvent::DormancyOnset));
    }

    // --- Chilling ---

    #[test]
    fn chilling_in_range() {
        assert_eq!(chilling_contribution(3.0), 1.0);
        assert_eq!(chilling_contribution(0.0), 1.0);
        assert_eq!(chilling_contribution(7.2), 1.0);
    }

    #[test]
    fn chilling_out_of_range() {
        assert_eq!(chilling_contribution(-1.0), 0.0);
        assert_eq!(chilling_contribution(8.0), 0.0);
        assert_eq!(chilling_contribution(20.0), 0.0);
    }

    #[test]
    fn accumulated_chill_basic() {
        // 4 hours in range, 2 out
        let temps = [3.0, 5.0, 0.0, 7.0, -1.0, 10.0];
        let chill = accumulated_chill(&temps);
        assert!((chill - 4.0).abs() < 0.01, "got {chill}");
    }

    #[test]
    fn accumulated_chill_empty() {
        assert_eq!(accumulated_chill(&[]), 0.0);
    }

    // --- Dormancy break ---

    #[test]
    fn dormancy_broken_sufficient_chill() {
        assert!(dormancy_broken(1200.0, 1000.0));
    }

    #[test]
    fn dormancy_not_broken_insufficient() {
        assert!(!dormancy_broken(500.0, 1000.0));
    }

    #[test]
    fn dormancy_broken_zero_requirement() {
        assert!(dormancy_broken(0.0, 0.0));
    }

    #[test]
    fn dormancy_broken_at_exact_threshold() {
        assert!(dormancy_broken(1000.0, 1000.0));
    }

    // --- Senescence ---

    #[test]
    fn senescence_short_days_and_cold() {
        assert!(senescence_triggered(11.0, 12.0, 8.0, 10.0));
    }

    #[test]
    fn senescence_not_triggered_long_days() {
        assert!(!senescence_triggered(14.0, 12.0, 8.0, 10.0));
    }

    #[test]
    fn senescence_not_triggered_warm() {
        assert!(!senescence_triggered(11.0, 12.0, 15.0, 10.0));
    }

    // --- Dormancy onset ---

    #[test]
    fn dormancy_onset_short_days() {
        assert!(dormancy_onset_triggered(9.0, 10.0, 5.0, -2.0));
    }

    #[test]
    fn dormancy_onset_frost() {
        assert!(dormancy_onset_triggered(12.0, 10.0, -5.0, -2.0));
    }

    #[test]
    fn dormancy_onset_not_triggered() {
        assert!(!dormancy_onset_triggered(12.0, 10.0, 5.0, -2.0));
    }

    // --- Phenological progress ---

    #[test]
    fn progress_zero_at_start() {
        assert_eq!(phenological_progress(0.0, PhenologicalEvent::BudBreak), 0.0);
    }

    #[test]
    fn progress_half() {
        let p = phenological_progress(75.0, PhenologicalEvent::BudBreak); // threshold=150
        assert!((p - 0.5).abs() < 0.01);
    }

    #[test]
    fn progress_capped_at_one() {
        assert_eq!(
            phenological_progress(1000.0, PhenologicalEvent::BudBreak),
            1.0
        );
    }

    #[test]
    fn progress_dormancy_returns_zero() {
        assert_eq!(
            phenological_progress(500.0, PhenologicalEvent::DormancyBreak),
            0.0
        );
    }

    // --- Cross-concept ---

    #[test]
    fn full_season_cycle() {
        // Winter: accumulate chill
        let winter_temps: Vec<f32> = (0..1200).map(|_| 3.0).collect();
        let chill = accumulated_chill(&winter_temps);
        assert!(
            dormancy_broken(chill, 1000.0),
            "1200h chill should break dormancy"
        );

        // Spring: accumulate GDD
        let spring_temps = [12.0; 60]; // 60 days at 12°C, base 5 → 7 GDD/day = 420
        let gdd = accumulated_gdd(&spring_temps, 5.0);
        assert!(event_reached(gdd, PhenologicalEvent::BudBreak));
        assert!(event_reached(gdd, PhenologicalEvent::LeafOut));
        assert!(!event_reached(gdd, PhenologicalEvent::Flowering));

        // Autumn: senescence
        assert!(senescence_triggered(11.0, 12.0, 8.0, 10.0));

        // Winter: dormancy
        assert!(dormancy_onset_triggered(9.0, 10.0, -3.0, -2.0));
    }
}
