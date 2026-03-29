//! Cross-crate primitive-value conversions for AGNOS science crates.
//!
//! Bridge functions accept f64 inputs (matching badal/ushma convention) and
//! return f32 outputs (matching vanaspati convention). No external dependencies —
//! all conversions use primitive arithmetic only.

use crate::mortality::frost_mortality;
use crate::photosynthesis::temperature_factor;
use crate::season::growth_modifier_at;

// ── Constants ─────────────────────────────────────────────────────────

/// Solar PAR fraction — ~45% of total solar irradiance is photosynthetically active.
const SOLAR_PAR_FRACTION: f64 = 0.45;

/// PAR conversion — 1 W/m² of PAR ≈ 4.57 µmol photons/m²/s.
const PAR_CONVERSION: f64 = 4.57;

/// Combined solar to PAR conversion (W/m² total solar → µmol photons/m²/s PAR).
const SOLAR_TO_PAR: f64 = SOLAR_PAR_FRACTION * PAR_CONVERSION;

/// Root activity optimum temperature (°C).
const ROOT_OPTIMUM_CELSIUS: f64 = 20.0;

/// Beer-Lambert extinction coefficient for canopy cover (dimensionless).
const CANOPY_EXTINCTION_K: f64 = 0.5;

// ── Badal bridges (weather/atmosphere) ────────────────────────────────

/// Convert total solar irradiance (W/m²) to PAR (µmol photons/m²/s).
///
/// ~45% of total solar is in the PAR band (400–700 nm);
/// 1 W/m² PAR ≈ 4.57 µmol photons/m²/s.
///
/// Connects to: `photosynthesis_rate(max_rate, quantum_yield, light_intensity)`
///
/// - `irradiance_w_m2` — total solar irradiance (W/m²)
#[must_use]
#[inline]
pub fn solar_to_par(irradiance_w_m2: f64) -> f32 {
    let par = irradiance_w_m2.max(0.0) * SOLAR_TO_PAR;
    tracing::trace!(irradiance_w_m2, par, "solar_to_par");
    par as f32
}

/// Convert atmospheric conditions to photosynthesis inputs.
///
/// Returns `(temperature_celsius, par_umol_m2_s)` ready for
/// `temperature_factor()` and `photosynthesis_rate()`.
///
/// - `temperature_k` — air temperature (Kelvin)
/// - `solar_irradiance_w_m2` — total solar irradiance (W/m²)
#[must_use]
pub fn atmosphere_to_photosynthesis_inputs(
    temperature_k: f64,
    solar_irradiance_w_m2: f64,
) -> (f32, f32) {
    let temp_c = (temperature_k - 273.15) as f32;
    let par = solar_to_par(solar_irradiance_w_m2);
    tracing::trace!(
        temperature_k,
        temp_c,
        par,
        "atmosphere_to_photosynthesis_inputs"
    );
    (temp_c, par)
}

/// Convert rainfall rate and duration to water supply (liters/m²).
///
/// 1 mm rainfall = 1 liter/m².
///
/// Connects to: `drought_mortality(water_available, water_needed)`
///
/// - `rate_mm_hr` — precipitation rate (mm/hr)
/// - `duration_hours` — duration (hours)
#[must_use]
#[inline]
pub fn rainfall_to_water_supply(rate_mm_hr: f64, duration_hours: f64) -> f32 {
    let supply = rate_mm_hr.max(0.0) * duration_hours.max(0.0);
    tracing::trace!(
        rate_mm_hr,
        duration_hours,
        supply,
        "rainfall_to_water_supply"
    );
    supply as f32
}

/// Convert frost risk probability and plant hardiness to frost mortality.
///
/// Combines atmospheric frost probability with plant-level vulnerability.
/// Returns `frost_mortality(temp, hardiness) × frost_risk`.
///
/// Connects to: `frost_mortality(temp_celsius, hardiness_celsius)`
///
/// - `temperature_celsius` — air temperature (°C)
/// - `frost_risk` — frost probability from badal (0.0–1.0)
/// - `hardiness_celsius` — plant cold hardiness threshold (°C)
#[must_use]
pub fn frost_risk_to_mortality(
    temperature_celsius: f64,
    frost_risk: f64,
    hardiness_celsius: f64,
) -> f32 {
    if frost_risk <= 0.0 {
        return 0.0;
    }
    let risk = frost_risk.clamp(0.0, 1.0) as f32;
    let mort = frost_mortality(temperature_celsius as f32, hardiness_celsius as f32);
    let result = risk * mort;
    tracing::trace!(
        temperature_celsius,
        frost_risk,
        hardiness_celsius,
        result,
        "frost_risk_to_mortality"
    );
    result
}

/// Determine if frost conditions should trigger plant dormancy.
///
/// Returns `true` if frost risk exceeds 0.5 and temperature is below
/// the dormancy trigger point.
///
/// Connects to: `GrowthStage::Dormant`
///
/// - `temperature_celsius` — air temperature (°C)
/// - `frost_risk` — frost probability from badal (0.0–1.0)
/// - `dormancy_threshold_celsius` — temperature below which plant goes dormant (°C)
#[must_use]
#[inline]
pub fn frost_to_dormancy(
    temperature_celsius: f64,
    frost_risk: f64,
    dormancy_threshold_celsius: f64,
) -> bool {
    let dormant = frost_risk > 0.5 && temperature_celsius < dormancy_threshold_celsius;
    tracing::trace!(
        temperature_celsius,
        frost_risk,
        dormancy_threshold_celsius,
        dormant,
        "frost_to_dormancy"
    );
    dormant
}

/// Convert wind speed at reference height to wind at seed release height (m/s).
///
/// Logarithmic wind profile: `V(z) = V_ref × ln(z/z0) / ln(z_ref/z0)`
///
/// Connects to: `dispersal_distance(method, seed_mass_g, release_height_m, wind_speed_m_s)`
///
/// - `speed_ref_ms` — reference wind speed (m/s)
/// - `z_ref_m` — reference measurement height (m)
/// - `z_release_m` — seed release height (m)
/// - `z0_m` — surface roughness length (m, forest ≈ 1.0, grass ≈ 0.03)
#[must_use]
pub fn wind_to_dispersal_speed(
    speed_ref_ms: f64,
    z_ref_m: f64,
    z_release_m: f64,
    z0_m: f64,
) -> f32 {
    if z0_m <= 0.0 || z_ref_m <= z0_m || z_release_m <= z0_m || speed_ref_ms <= 0.0 {
        return 0.0;
    }
    let ln_ref = (z_ref_m / z0_m).ln();
    let ln_release = (z_release_m / z0_m).ln();
    let speed = speed_ref_ms * ln_release / ln_ref;
    tracing::trace!(
        speed_ref_ms,
        z_ref_m,
        z_release_m,
        z0_m,
        speed,
        "wind_to_dispersal_speed"
    );
    speed.max(0.0) as f32
}

/// Composite growth rate multiplier (0.0–1.0) from atmospheric conditions.
///
/// `multiplier = temperature_factor × light_factor × growth_modifier_at`
///
/// Light factor: `min(1.0, PAR / 800.0)` where 800 µmol/m²/s is full-sun
/// saturation for typical C3 plants.
///
/// Connects to: `GrowthModel::daily_growth()` (multiply result by this)
///
/// - `temperature_celsius` — air temperature (°C)
/// - `optimum_celsius` — species optimal temperature (°C)
/// - `solar_irradiance_w_m2` — total solar irradiance (W/m²)
/// - `day_of_year` — day (1–365)
/// - `latitude_deg` — latitude (degrees)
#[must_use]
pub fn growing_conditions_to_growth_multiplier(
    temperature_celsius: f64,
    optimum_celsius: f64,
    solar_irradiance_w_m2: f64,
    day_of_year: u16,
    latitude_deg: f64,
) -> f32 {
    let temp_f = temperature_factor(temperature_celsius as f32, optimum_celsius as f32);
    let par = solar_to_par(solar_irradiance_w_m2);
    let light_f = (par / 800.0).clamp(0.0, 1.0);
    let season_f = growth_modifier_at(day_of_year, latitude_deg as f32);
    let multiplier = temp_f * light_f * season_f;
    tracing::trace!(
        temperature_celsius,
        optimum_celsius,
        solar_irradiance_w_m2,
        temp_f,
        light_f,
        season_f,
        multiplier,
        "growing_conditions_to_growth_multiplier"
    );
    multiplier
}

// ── Ushma bridges (thermodynamics) ────────────────────────────────────

/// Soil temperature to root activity scaling factor (0.0–1.0).
///
/// Gaussian bell curve with optimum at 20°C, σ² = 200.
/// Below 5°C roots are nearly inactive; above 35°C heat stress reduces activity.
///
/// Connects to: `RootSystem::water_uptake_rate` (scale by this factor)
///
/// - `soil_temperature_k` — soil temperature (Kelvin)
#[must_use]
#[inline]
pub fn soil_temperature_to_root_activity(soil_temperature_k: f64) -> f32 {
    let t_c = soil_temperature_k - 273.15;
    let diff = t_c - ROOT_OPTIMUM_CELSIUS;
    let activity = (-diff * diff / 200.0).exp();
    tracing::trace!(
        soil_temperature_k,
        t_c,
        activity,
        "soil_temperature_to_root_activity"
    );
    activity as f32
}

/// Soil temperature to growth factor (0.0–1.0).
///
/// Returns 0.0 if soil is frozen (< 273.15 K), otherwise returns root activity.
///
/// Connects to: `GrowthModel::daily_growth()` (multiply by this)
///
/// - `soil_temperature_k` — soil temperature (Kelvin)
#[must_use]
#[inline]
pub fn soil_temperature_to_growth_factor(soil_temperature_k: f64) -> f32 {
    if soil_temperature_k < 273.15 {
        tracing::trace!(
            soil_temperature_k,
            factor = 0.0,
            "soil_temperature_to_growth_factor (frozen)"
        );
        return 0.0;
    }
    soil_temperature_to_root_activity(soil_temperature_k)
}

/// Leaf cooling from evapotranspiration (°C reduction).
///
/// Transpiration cools leaves below air temperature.
/// Simplified: `ΔT ≈ ET_rate × 2.5` (°C per mm/hr), capped at 15°C.
///
/// Connects to: subtract from air temperature before `temperature_factor()`
///
/// - `et_rate_mm_hr` — evapotranspiration rate (mm/hr)
#[must_use]
#[inline]
pub fn evapotranspiration_cooling(et_rate_mm_hr: f64) -> f32 {
    let cooling = (et_rate_mm_hr.max(0.0) * 2.5).min(15.0);
    tracing::trace!(et_rate_mm_hr, cooling, "evapotranspiration_cooling");
    cooling as f32
}

/// Wet-bulb temperature to plant heat stress factor (0.0–1.0).
///
/// High wet-bulb temperatures impair transpirational cooling.
/// Stress onset at 28°C, severe at 35°C.
/// `stress = ((T_wb - 28) / 7).clamp(0, 1)`
///
/// Consumers subtract from 1.0 for a multiplier: `effective = 1.0 - stress`.
///
/// - `wet_bulb_k` — wet-bulb temperature (Kelvin)
#[must_use]
#[inline]
pub fn wet_bulb_to_heat_stress(wet_bulb_k: f64) -> f32 {
    let t_c = wet_bulb_k - 273.15;
    let stress = ((t_c - 28.0) / 7.0).clamp(0.0, 1.0);
    tracing::trace!(wet_bulb_k, t_c, stress, "wet_bulb_to_heat_stress");
    stress as f32
}

// ── Jantu bridges (creature behavior) ─────────────────────────────────

/// Canopy leaf area index to habitat cover score (0.0–1.0).
///
/// Beer-Lambert extinction: `cover = 1 - e^(-k × LAI)`, k = 0.5.
/// Dense canopy provides better cover for creatures.
///
/// Connects to: `height_to_leaf_area()` via LAI computation
///
/// - `leaf_area_index` — LAI (m² leaf per m² ground, dimensionless)
#[must_use]
#[inline]
pub fn canopy_to_habitat_score(leaf_area_index: f64) -> f32 {
    let score = 1.0 - (-CANOPY_EXTINCTION_K * leaf_area_index.max(0.0)).exp();
    tracing::trace!(leaf_area_index, score, "canopy_to_habitat_score");
    score as f32
}

/// Reproductive biomass to food availability score (0.0–1.0).
///
/// Logarithmic saturation: `score = 1 - e^(-seed_count / 1000)`.
/// Many small seeds or fewer large fruits both provide food.
///
/// Connects to: `BiomassPool::reproductive_kg` and `SeedProfile::mass_g`
///
/// - `reproductive_biomass_kg` — reproductive organ mass (kilograms)
/// - `seed_mass_g` — individual seed/fruit mass (grams)
#[must_use]
pub fn seed_production_to_food(reproductive_biomass_kg: f64, seed_mass_g: f64) -> f32 {
    if reproductive_biomass_kg <= 0.0 || seed_mass_g <= 0.0 {
        return 0.0;
    }
    let seed_count = reproductive_biomass_kg * 1000.0 / seed_mass_g;
    let score = (1.0 - (-seed_count / 1000.0).exp()).min(1.0);
    tracing::trace!(
        reproductive_biomass_kg,
        seed_mass_g,
        seed_count,
        score,
        "seed_production_to_food"
    );
    score as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Badal bridge tests ────────────────────────────────────────────

    #[test]
    fn solar_to_par_full_sun() {
        let par = solar_to_par(1000.0);
        // 1000 × 0.45 × 4.57 ≈ 2057
        assert!((par - 2056.5).abs() < 1.0, "got {par}");
    }

    #[test]
    fn solar_to_par_zero() {
        assert_eq!(solar_to_par(0.0), 0.0);
    }

    #[test]
    fn solar_to_par_negative_clamped() {
        assert_eq!(solar_to_par(-100.0), 0.0);
    }

    #[test]
    fn atmosphere_to_photosynthesis_summer() {
        let (temp, par) = atmosphere_to_photosynthesis_inputs(298.15, 800.0);
        assert!((temp - 25.0).abs() < 0.1);
        assert!(par > 1600.0);
    }

    #[test]
    fn atmosphere_to_photosynthesis_night() {
        let (temp, par) = atmosphere_to_photosynthesis_inputs(283.15, 0.0);
        assert!((temp - 10.0).abs() < 0.1);
        assert_eq!(par, 0.0);
    }

    #[test]
    fn rainfall_basic() {
        let supply = rainfall_to_water_supply(5.0, 2.0);
        assert!((supply - 10.0).abs() < 0.01);
    }

    #[test]
    fn rainfall_zero() {
        assert_eq!(rainfall_to_water_supply(0.0, 5.0), 0.0);
    }

    #[test]
    fn rainfall_negative_clamped() {
        assert_eq!(rainfall_to_water_supply(-3.0, 2.0), 0.0);
    }

    #[test]
    fn frost_risk_warm() {
        assert_eq!(frost_risk_to_mortality(10.0, 0.0, -10.0), 0.0);
    }

    #[test]
    fn frost_risk_cold() {
        let m = frost_risk_to_mortality(-15.0, 0.8, -10.0);
        assert!(
            m > 0.5,
            "cold + high risk should produce high mortality, got {m}"
        );
    }

    #[test]
    fn frost_risk_no_risk() {
        let m = frost_risk_to_mortality(-15.0, 0.0, -10.0);
        assert_eq!(m, 0.0, "zero frost risk → zero mortality");
    }

    #[test]
    fn dormancy_triggers() {
        assert!(frost_to_dormancy(-5.0, 0.7, 0.0));
    }

    #[test]
    fn dormancy_warm() {
        assert!(!frost_to_dormancy(10.0, 0.1, 0.0));
    }

    #[test]
    fn dormancy_low_risk() {
        assert!(!frost_to_dormancy(-5.0, 0.3, 0.0));
    }

    #[test]
    fn wind_same_height() {
        let s = wind_to_dispersal_speed(10.0, 10.0, 10.0, 1.0);
        assert!((s - 10.0).abs() < 0.01);
    }

    #[test]
    fn wind_higher() {
        let s = wind_to_dispersal_speed(10.0, 10.0, 20.0, 1.0);
        assert!(s > 10.0, "higher release → faster wind");
    }

    #[test]
    fn wind_lower() {
        let s = wind_to_dispersal_speed(10.0, 10.0, 5.0, 1.0);
        assert!(s < 10.0, "lower release → slower wind");
    }

    #[test]
    fn wind_invalid_z0() {
        assert_eq!(wind_to_dispersal_speed(10.0, 10.0, 20.0, 0.0), 0.0);
    }

    #[test]
    fn wind_zero_speed() {
        assert_eq!(wind_to_dispersal_speed(0.0, 10.0, 20.0, 1.0), 0.0);
    }

    #[test]
    fn growing_conditions_summer_optimal() {
        let m = growing_conditions_to_growth_multiplier(25.0, 25.0, 800.0, 172, 45.0);
        assert!(
            m > 0.7,
            "optimal summer conditions should give high multiplier, got {m}"
        );
    }

    #[test]
    fn growing_conditions_winter() {
        let m = growing_conditions_to_growth_multiplier(0.0, 25.0, 100.0, 356, 45.0);
        assert!(
            m < 0.1,
            "winter conditions should give low multiplier, got {m}"
        );
    }

    #[test]
    fn growing_conditions_night() {
        let m = growing_conditions_to_growth_multiplier(25.0, 25.0, 0.0, 172, 45.0);
        assert_eq!(m, 0.0, "no light → zero growth");
    }

    // ── Ushma bridge tests ────────────────────────────────────────────

    #[test]
    fn soil_temp_optimum() {
        let a = soil_temperature_to_root_activity(293.15); // 20°C
        assert!((a - 1.0).abs() < 0.01, "optimum should be ~1.0, got {a}");
    }

    #[test]
    fn soil_temp_frozen() {
        let a = soil_temperature_to_root_activity(268.15); // -5°C
        assert!(a < 0.1, "frozen soil should have low activity, got {a}");
    }

    #[test]
    fn soil_temp_hot() {
        let a = soil_temperature_to_root_activity(318.15); // 45°C
        assert!(a < 0.3, "hot soil should reduce activity, got {a}");
    }

    #[test]
    fn soil_growth_frozen() {
        assert_eq!(soil_temperature_to_growth_factor(270.0), 0.0);
    }

    #[test]
    fn soil_growth_warm() {
        let f = soil_temperature_to_growth_factor(293.15);
        assert!(f > 0.9);
    }

    #[test]
    fn et_cooling_typical() {
        let c = evapotranspiration_cooling(3.0);
        assert!((c - 7.5).abs() < 0.01);
    }

    #[test]
    fn et_cooling_zero() {
        assert_eq!(evapotranspiration_cooling(0.0), 0.0);
    }

    #[test]
    fn et_cooling_capped() {
        let c = evapotranspiration_cooling(100.0);
        assert_eq!(c, 15.0);
    }

    #[test]
    fn et_cooling_negative() {
        assert_eq!(evapotranspiration_cooling(-5.0), 0.0);
    }

    #[test]
    fn wet_bulb_no_stress() {
        let s = wet_bulb_to_heat_stress(295.15); // 22°C
        assert_eq!(s, 0.0);
    }

    #[test]
    fn wet_bulb_severe() {
        let s = wet_bulb_to_heat_stress(308.15); // 35°C
        assert_eq!(s, 1.0);
    }

    #[test]
    fn wet_bulb_onset() {
        let s = wet_bulb_to_heat_stress(301.15); // 28°C
        assert!(s < 0.01, "at onset, stress should be ~0, got {s}");
    }

    // ── Jantu bridge tests ────────────────────────────────────────────

    #[test]
    fn canopy_zero_lai() {
        let s = canopy_to_habitat_score(0.0);
        assert!(s.abs() < 0.01);
    }

    #[test]
    fn canopy_high_lai() {
        let s = canopy_to_habitat_score(6.0);
        assert!(s > 0.9, "dense canopy should give high cover, got {s}");
    }

    #[test]
    fn canopy_moderate_lai() {
        let s = canopy_to_habitat_score(3.0);
        assert!(s > 0.7 && s < 0.85, "LAI=3 should give ~0.78, got {s}");
    }

    #[test]
    fn canopy_negative_clamped() {
        assert_eq!(canopy_to_habitat_score(-1.0), 0.0);
    }

    #[test]
    fn seed_production_none() {
        assert_eq!(seed_production_to_food(0.0, 1.0), 0.0);
    }

    #[test]
    fn seed_production_moderate() {
        let s = seed_production_to_food(1.0, 1.0); // 1000 seeds
        assert!(s > 0.5 && s < 0.7, "1000 seeds should give ~0.63, got {s}");
    }

    #[test]
    fn seed_production_heavy() {
        let s = seed_production_to_food(10.0, 0.1); // 100000 seeds
        assert!(s > 0.99, "many seeds should saturate, got {s}");
    }

    #[test]
    fn seed_production_zero_mass() {
        assert_eq!(seed_production_to_food(1.0, 0.0), 0.0);
    }

    #[test]
    fn seed_production_negative() {
        assert_eq!(seed_production_to_food(-1.0, 1.0), 0.0);
    }
}
