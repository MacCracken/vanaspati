//! Dynamic leaf area index — seasonal LAI changes, stress-induced leaf loss,
//! and the connection between leaf biomass and canopy light interception.
//!
//! LAI (m² leaf / m² ground) drives both photosynthesis (light interception)
//! and transpiration (stomatal area). It varies seasonally and responds to
//! drought, frost, and herbivory.

use serde::{Deserialize, Serialize};

/// Leaf habit — determines seasonal LAI dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LeafHabit {
    /// Deciduous — full leaf loss in winter, regrowth in spring.
    Deciduous,
    /// Evergreen — gradual turnover, maintains ~70% LAI year-round.
    Evergreen,
    /// Drought-deciduous — drops leaves during dry season.
    DroughtDeciduous,
}

/// LAI from leaf biomass and specific leaf area.
///
/// `LAI = leaf_biomass_kg × SLA / ground_area_m2`
///
/// SLA (specific leaf area) = leaf area per unit dry mass (m²/kg).
/// Typical values:
/// - Broadleaf deciduous: 20–30 m²/kg
/// - Conifer: 5–10 m²/kg
/// - Grass: 25–35 m²/kg
///
/// - `leaf_biomass_kg` — leaf dry mass (kg)
/// - `sla_m2_per_kg` — specific leaf area (m² leaf / kg leaf)
/// - `ground_area_m2` — ground area occupied by crown (m²)
#[must_use]
#[inline]
pub fn lai_from_biomass(leaf_biomass_kg: f32, sla_m2_per_kg: f32, ground_area_m2: f32) -> f32 {
    if leaf_biomass_kg <= 0.0 || sla_m2_per_kg <= 0.0 || ground_area_m2 <= 0.0 {
        return 0.0;
    }
    let lai = leaf_biomass_kg * sla_m2_per_kg / ground_area_m2;
    tracing::trace!(
        leaf_biomass_kg,
        sla_m2_per_kg,
        ground_area_m2,
        lai,
        "lai_from_biomass"
    );
    lai
}

/// Seasonal LAI multiplier (0.0–1.0).
///
/// Modulates LAI based on leaf habit and day of year.
///
/// - Deciduous: follows a smooth seasonal curve — 0.0 in winter, 1.0 in summer
/// - Evergreen: minimum 0.7, slight reduction in winter
/// - Drought-deciduous: based on water availability, not day of year
///
/// For deciduous, uses a sine-based phenology curve:
/// `multiplier = max(0, sin(π × (day - leaf_on) / (leaf_off - leaf_on)))`
///
/// - `habit` — leaf habit type
/// - `day_of_year` — day (1–365)
/// - `latitude_deg` — latitude (degrees, for hemisphere)
#[must_use]
pub fn seasonal_lai_multiplier(habit: LeafHabit, day_of_year: u16, latitude_deg: f32) -> f32 {
    let day = day_of_year.clamp(1, 365) as f32;

    let mult = match habit {
        LeafHabit::Deciduous => {
            // Leaf-on and leaf-off days depend on hemisphere
            let (leaf_on, leaf_off) = if latitude_deg >= 0.0 {
                (100.0, 300.0) // NH: ~April 10 to ~October 27
            } else {
                (280.0, 115.0) // SH: shifted by 6 months (wraps around)
            };

            if latitude_deg >= 0.0 {
                if day < leaf_on || day > leaf_off {
                    0.0
                } else {
                    let progress = (day - leaf_on) / (leaf_off - leaf_on);
                    (std::f32::consts::PI * progress).sin().max(0.0)
                }
            } else {
                // Southern hemisphere: leaf-on at day 280, off at day 115 (wraps)
                let adjusted = if day >= leaf_on {
                    day - leaf_on
                } else if day <= leaf_off {
                    day + 365.0 - leaf_on
                } else {
                    return 0.0;
                };
                let season_length = 365.0 - leaf_on + leaf_off;
                let progress = adjusted / season_length;
                (std::f32::consts::PI * progress).sin().max(0.0)
            }
        }
        LeafHabit::Evergreen => {
            // Slight seasonal reduction: minimum 0.7 in winter
            let seasonal = (2.0 * std::f32::consts::PI * (day - 172.0) / 365.0).cos();
            let adjusted = if latitude_deg < 0.0 {
                -seasonal
            } else {
                seasonal
            };
            0.85 + 0.15 * adjusted // range: 0.70–1.00
        }
        LeafHabit::DroughtDeciduous => {
            // Controlled by water, not day — return 1.0 (caller applies water stress)
            1.0
        }
    };

    tracing::trace!(
        ?habit,
        day_of_year,
        latitude_deg,
        mult,
        "seasonal_lai_multiplier"
    );
    mult
}

/// Water stress leaf loss factor (0.0–1.0).
///
/// Plants shed leaves under drought to reduce water demand.
/// Drought-deciduous species shed aggressively; evergreens resist.
///
/// `retention = rwc^exponent`
///
/// Exponent by habit:
/// - Drought-deciduous: 2.0 (drops leaves quickly under mild drought)
/// - Deciduous: 1.0 (linear with water stress)
/// - Evergreen: 0.5 (resists leaf loss until severe drought)
///
/// - `relative_water_content` — soil RWC (0.0–1.0)
/// - `habit` — leaf habit type
#[must_use]
#[inline]
pub fn drought_leaf_retention(relative_water_content: f32, habit: LeafHabit) -> f32 {
    let rwc = relative_water_content.clamp(0.0, 1.0);
    let exponent = match habit {
        LeafHabit::DroughtDeciduous => 2.0,
        LeafHabit::Deciduous => 1.0,
        LeafHabit::Evergreen => 0.5,
    };
    let retention = rwc.powf(exponent);
    tracing::trace!(
        relative_water_content,
        ?habit,
        exponent,
        retention,
        "drought_leaf_retention"
    );
    retention
}

/// Frost-induced leaf loss fraction (0.0–1.0).
///
/// Hard frost kills exposed leaves. Returns fraction of leaves killed.
///
/// `killed = max(0, (frost_threshold - temp) / 10)²`
///
/// Below the frost threshold, damage increases quadratically.
/// Typical thresholds: deciduous -2°C, evergreen -5°C.
///
/// - `temp_celsius` — minimum daily temperature (°C)
/// - `frost_threshold_celsius` — temperature below which leaf damage occurs (°C)
#[must_use]
#[inline]
pub fn frost_leaf_loss(temp_celsius: f32, frost_threshold_celsius: f32) -> f32 {
    if temp_celsius >= frost_threshold_celsius {
        return 0.0;
    }
    let damage = ((frost_threshold_celsius - temp_celsius) / 10.0).min(1.0);
    let killed = (damage * damage).clamp(0.0, 1.0);
    tracing::trace!(
        temp_celsius,
        frost_threshold_celsius,
        killed,
        "frost_leaf_loss"
    );
    killed
}

/// Maximum LAI constraint for a species (m² leaf / m² ground).
///
/// Species-specific upper bound on LAI, determined by crown architecture
/// and self-shading. Beyond this, additional leaves gain no light.
///
/// Typical values:
/// - Broadleaf deciduous: 6–8
/// - Conifer: 8–12
/// - Grassland: 3–5
/// - Tropical forest: 7–10
///
/// - `is_conifer` — true for needle-leaved species
/// - `is_tropical` — true for tropical species
#[must_use]
#[inline]
pub fn max_lai(is_conifer: bool, is_tropical: bool) -> f32 {
    match (is_conifer, is_tropical) {
        (true, _) => 10.0,
        (false, true) => 8.0,
        (false, false) => 7.0,
    }
}

/// Effective LAI combining all factors.
///
/// `LAI_eff = min(LAI_max, LAI_biomass × seasonal × drought_retention × (1 - frost_loss))`
///
/// - `lai_biomass` — LAI from leaf biomass (m²/m²)
/// - `lai_max` — species maximum LAI (m²/m²)
/// - `seasonal` — seasonal multiplier (0.0–1.0)
/// - `drought_retention` — drought leaf retention (0.0–1.0)
/// - `frost_loss_fraction` — frost-killed leaf fraction (0.0–1.0)
#[must_use]
#[inline]
pub fn effective_lai(
    lai_biomass: f32,
    lai_max: f32,
    seasonal: f32,
    drought_retention: f32,
    frost_loss_fraction: f32,
) -> f32 {
    let effective = lai_biomass
        * seasonal.clamp(0.0, 1.0)
        * drought_retention.clamp(0.0, 1.0)
        * (1.0 - frost_loss_fraction.clamp(0.0, 1.0));
    let clamped = effective.clamp(0.0, lai_max.max(0.0));
    tracing::trace!(
        lai_biomass,
        lai_max,
        seasonal,
        drought_retention,
        frost_loss_fraction,
        effective = clamped,
        "effective_lai"
    );
    clamped
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- lai_from_biomass ---

    #[test]
    fn lai_from_biomass_basic() {
        // 50 kg leaves × 25 m²/kg / 100 m² ground = 12.5
        let lai = lai_from_biomass(50.0, 25.0, 100.0);
        assert!((lai - 12.5).abs() < 0.01, "got {lai}");
    }

    #[test]
    fn lai_from_biomass_zero_leaves() {
        assert_eq!(lai_from_biomass(0.0, 25.0, 100.0), 0.0);
    }

    #[test]
    fn lai_from_biomass_zero_area() {
        assert_eq!(lai_from_biomass(50.0, 25.0, 0.0), 0.0);
    }

    // --- seasonal LAI ---

    #[test]
    fn deciduous_peak_in_summer() {
        let summer = seasonal_lai_multiplier(LeafHabit::Deciduous, 200, 45.0);
        assert!(summer > 0.9, "mid-summer should be near 1.0, got {summer}");
    }

    #[test]
    fn deciduous_zero_in_winter() {
        let winter = seasonal_lai_multiplier(LeafHabit::Deciduous, 15, 45.0);
        assert_eq!(winter, 0.0, "January should be leafless");
    }

    #[test]
    fn evergreen_always_above_threshold() {
        for day in (1..=365).step_by(30) {
            let m = seasonal_lai_multiplier(LeafHabit::Evergreen, day, 45.0);
            assert!(m >= 0.69, "day {day}: {m}");
        }
    }

    #[test]
    fn drought_deciduous_always_one() {
        let m = seasonal_lai_multiplier(LeafHabit::DroughtDeciduous, 200, 45.0);
        assert_eq!(m, 1.0);
    }

    #[test]
    fn southern_hemisphere_shifted() {
        // January = summer in SH
        let sh_jan = seasonal_lai_multiplier(LeafHabit::Deciduous, 15, -35.0);
        let nh_jan = seasonal_lai_multiplier(LeafHabit::Deciduous, 15, 45.0);
        assert!(sh_jan > nh_jan, "SH January should be leafy: sh={sh_jan}");
    }

    // --- drought leaf retention ---

    #[test]
    fn drought_retention_full_water() {
        for habit in [
            LeafHabit::Deciduous,
            LeafHabit::Evergreen,
            LeafHabit::DroughtDeciduous,
        ] {
            assert_eq!(drought_leaf_retention(1.0, habit), 1.0, "{habit:?}");
        }
    }

    #[test]
    fn drought_deciduous_sheds_fastest() {
        let rwc = 0.5;
        let dd = drought_leaf_retention(rwc, LeafHabit::DroughtDeciduous);
        let ev = drought_leaf_retention(rwc, LeafHabit::Evergreen);
        assert!(
            dd < ev,
            "drought-deciduous should shed more: dd={dd}, ev={ev}"
        );
    }

    #[test]
    fn drought_retention_zero_water() {
        assert_eq!(drought_leaf_retention(0.0, LeafHabit::Deciduous), 0.0);
    }

    // --- frost leaf loss ---

    #[test]
    fn frost_no_damage_above_threshold() {
        assert_eq!(frost_leaf_loss(5.0, -2.0), 0.0);
    }

    #[test]
    fn frost_damage_below_threshold() {
        let killed = frost_leaf_loss(-5.0, -2.0);
        assert!(killed > 0.0, "below threshold should cause damage");
        assert!(killed < 0.5, "moderate frost shouldn't kill all leaves");
    }

    #[test]
    fn frost_severe_high_loss() {
        let killed = frost_leaf_loss(-15.0, -2.0);
        assert!(
            killed > 0.8,
            "severe frost should kill most leaves, got {killed}"
        );
    }

    // --- max LAI ---

    #[test]
    fn conifer_highest_max_lai() {
        assert!(max_lai(true, false) > max_lai(false, false));
    }

    // --- effective LAI ---

    #[test]
    fn effective_lai_full_conditions() {
        let eff = effective_lai(6.0, 8.0, 1.0, 1.0, 0.0);
        assert!((eff - 6.0).abs() < 0.01);
    }

    #[test]
    fn effective_lai_capped_at_max() {
        let eff = effective_lai(15.0, 8.0, 1.0, 1.0, 0.0);
        assert!((eff - 8.0).abs() < 0.01);
    }

    #[test]
    fn effective_lai_winter_deciduous() {
        let eff = effective_lai(6.0, 8.0, 0.0, 1.0, 0.0);
        assert_eq!(eff, 0.0, "winter deciduous = no LAI");
    }

    #[test]
    fn effective_lai_drought_reduces() {
        let wet = effective_lai(6.0, 8.0, 1.0, 1.0, 0.0);
        let dry = effective_lai(6.0, 8.0, 1.0, 0.5, 0.0);
        assert!(dry < wet);
    }

    #[test]
    fn effective_lai_frost_reduces() {
        let safe = effective_lai(6.0, 8.0, 1.0, 1.0, 0.0);
        let frost = effective_lai(6.0, 8.0, 1.0, 1.0, 0.3);
        assert!(frost < safe);
    }
}
