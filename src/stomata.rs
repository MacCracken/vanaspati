//! Stomatal conductance — the valve controlling gas exchange between
//! leaf interior and atmosphere. Stomata regulate the trade-off between
//! CO₂ uptake (photosynthesis) and water loss (transpiration).

use hisab::transforms::inverse_lerp;
use serde::{Deserialize, Serialize};

/// Stomatal behavior type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StomatalBehavior {
    /// Anisohydric — stomata stay open under drought, prioritizing carbon gain.
    /// Risk: hydraulic failure. Examples: oak, sunflower.
    Anisohydric,
    /// Isohydric — stomata close early under drought, conserving water.
    /// Risk: carbon starvation. Examples: pine, maize.
    Isohydric,
}

/// Vapor pressure deficit (kPa).
///
/// `VPD = e_s - e_a`
///
/// The driving force for transpiration. High VPD = dry air = more water loss.
///
/// - `saturation_vp_kpa` — saturation vapor pressure at leaf temperature (kPa)
/// - `actual_vp_kpa` — actual ambient vapor pressure (kPa)
#[must_use]
#[inline]
pub fn vapor_pressure_deficit(saturation_vp_kpa: f32, actual_vp_kpa: f32) -> f32 {
    let vpd = (saturation_vp_kpa - actual_vp_kpa).max(0.0);
    tracing::trace!(
        saturation_vp_kpa,
        actual_vp_kpa,
        vpd,
        "vapor_pressure_deficit"
    );
    vpd
}

/// Saturation vapor pressure from temperature (kPa).
///
/// Magnus-Tetens formula (Bolton 1980):
/// `e_s = 0.6112 × e^(17.67 × T / (T + 243.5))`
///
/// - `temp_celsius` — temperature (°C)
#[must_use]
#[inline]
pub fn saturation_vapor_pressure(temp_celsius: f32) -> f32 {
    let es = 0.6112 * (17.67 * temp_celsius / (temp_celsius + 243.5)).exp();
    tracing::trace!(temp_celsius, es, "saturation_vapor_pressure");
    es
}

/// Ball-Berry stomatal conductance model (mol H₂O/m²/s).
///
/// `g_s = g0 + g1 × (A × RH / C_s)`
///
/// Where:
/// - g_s = stomatal conductance (mol H₂O/m²/s)
/// - g0 = minimum conductance when stomata are closed (mol/m²/s)
/// - g1 = slope parameter (dimensionless sensitivity)
/// - A = net photosynthesis rate (µmol CO₂/m²/s)
/// - RH = relative humidity at leaf surface (fraction, 0.0–1.0)
/// - C_s = CO₂ concentration at leaf surface (µmol/mol, ppm)
///
/// Typical values:
/// - g0: 0.01–0.04 mol/m²/s
/// - g1: 6–12 (C3 plants), 3–6 (C4 plants)
///
/// - `min_conductance` — g0 (mol H₂O/m²/s)
/// - `slope` — g1 (dimensionless)
/// - `photosynthesis_rate` — A (µmol CO₂/m²/s)
/// - `humidity_fraction` — RH at leaf surface (0.0–1.0)
/// - `co2_ppm` — CO₂ at leaf surface (µmol/mol)
#[must_use]
pub fn ball_berry_conductance(
    min_conductance: f32,
    slope: f32,
    photosynthesis_rate: f32,
    humidity_fraction: f32,
    co2_ppm: f32,
) -> f32 {
    if co2_ppm <= 0.0 || photosynthesis_rate <= 0.0 {
        return min_conductance.max(0.0);
    }
    let rh = humidity_fraction.clamp(0.0, 1.0);
    let gs = min_conductance + slope * (photosynthesis_rate * rh / co2_ppm);
    let result = gs.max(0.0);
    tracing::trace!(
        min_conductance,
        slope,
        photosynthesis_rate,
        humidity_fraction,
        co2_ppm,
        result,
        "ball_berry_conductance"
    );
    result
}

/// Transpiration rate from stomatal conductance (mmol H₂O/m²/s).
///
/// `E = g_s × VPD / P`
///
/// - `conductance_mol_m2_s` — stomatal conductance (mol H₂O/m²/s)
/// - `vpd_kpa` — vapor pressure deficit (kPa)
/// - `atmospheric_pressure_kpa` — atmospheric pressure (kPa, typically ~101.3)
#[must_use]
#[inline]
pub fn transpiration_rate(
    conductance_mol_m2_s: f32,
    vpd_kpa: f32,
    atmospheric_pressure_kpa: f32,
) -> f32 {
    if conductance_mol_m2_s <= 0.0 || vpd_kpa <= 0.0 || atmospheric_pressure_kpa <= 0.0 {
        return 0.0;
    }
    // E in mol/m²/s, convert to mmol/m²/s
    let e = conductance_mol_m2_s * vpd_kpa / atmospheric_pressure_kpa * 1000.0;
    tracing::trace!(
        conductance_mol_m2_s,
        vpd_kpa,
        atmospheric_pressure_kpa,
        e,
        "transpiration_rate"
    );
    e
}

/// Water use efficiency from photosynthesis and transpiration.
///
/// `WUE = A / E`
///
/// Instantaneous WUE: µmol CO₂ gained per mmol H₂O lost.
/// Typical C3: 3–6 µmol/mmol, C4: 6–12 µmol/mmol.
///
/// - `photosynthesis_umol` — net photosynthesis (µmol CO₂/m²/s)
/// - `transpiration_mmol` — transpiration (mmol H₂O/m²/s)
#[must_use]
#[inline]
pub fn instantaneous_wue(photosynthesis_umol: f32, transpiration_mmol: f32) -> f32 {
    if transpiration_mmol <= 0.0 {
        return 0.0;
    }
    let wue = photosynthesis_umol / transpiration_mmol;
    tracing::trace!(
        photosynthesis_umol,
        transpiration_mmol,
        wue,
        "instantaneous_wue"
    );
    wue
}

/// Drought stress factor on stomatal conductance (0.0–1.0).
///
/// Reduces conductance as soil water drops below a threshold.
/// Isohydric plants close stomata earlier (steeper curve).
///
/// `factor = ((water - wilting) / (field_cap - wilting))^p`
///
/// where p = 1.0 for anisohydric, p = 2.0 for isohydric (steeper closure).
///
/// - `soil_water_fraction` — current soil moisture (0.0–1.0)
/// - `wilting_point` — permanent wilting point (fraction, typically ~0.15)
/// - `field_capacity` — field capacity (fraction, typically ~0.35)
/// - `behavior` — stomatal behavior type
#[must_use]
pub fn drought_stomatal_factor(
    soil_water_fraction: f32,
    wilting_point: f32,
    field_capacity: f32,
    behavior: StomatalBehavior,
) -> f32 {
    if field_capacity <= wilting_point {
        return 0.0;
    }
    if soil_water_fraction <= wilting_point {
        return 0.0;
    }
    if soil_water_fraction >= field_capacity {
        return 1.0;
    }
    let relative = inverse_lerp(wilting_point, field_capacity, soil_water_fraction);
    let exponent = match behavior {
        StomatalBehavior::Anisohydric => 1.0,
        StomatalBehavior::Isohydric => 2.0,
    };
    let factor = relative.powf(exponent);
    tracing::trace!(
        soil_water_fraction,
        wilting_point,
        field_capacity,
        ?behavior,
        factor,
        "drought_stomatal_factor"
    );
    factor
}

/// VPD-driven stomatal reduction factor (0.0–1.0).
///
/// High VPD causes partial stomatal closure to limit water loss.
///
/// `factor = 1 / (1 + VPD / D0)`
///
/// D0 = sensitivity parameter (kPa). Lower D0 = more sensitive.
/// Typical D0: 1.0–3.0 kPa.
///
/// - `vpd_kpa` — vapor pressure deficit (kPa)
/// - `sensitivity_d0_kpa` — half-response VPD (kPa, conductance halves at this VPD)
#[must_use]
#[inline]
pub fn vpd_stomatal_factor(vpd_kpa: f32, sensitivity_d0_kpa: f32) -> f32 {
    if vpd_kpa <= 0.0 {
        return 1.0;
    }
    if sensitivity_d0_kpa <= 0.0 {
        return 0.0;
    }
    let factor = 1.0 / (1.0 + vpd_kpa / sensitivity_d0_kpa);
    tracing::trace!(vpd_kpa, sensitivity_d0_kpa, factor, "vpd_stomatal_factor");
    factor
}

/// Leaf boundary layer conductance (mol H₂O/m²/s).
///
/// Boundary layer limits gas exchange even when stomata are open.
/// Conductance increases with wind speed and decreases with leaf size.
///
/// `g_b = 0.147 × sqrt(wind_speed / leaf_width)`
///
/// Empirical formula for forced convection (Jones 2014).
///
/// - `wind_speed_m_s` — wind speed at leaf surface (m/s)
/// - `leaf_width_m` — characteristic leaf dimension (meters)
#[must_use]
#[inline]
pub fn boundary_layer_conductance(wind_speed_m_s: f32, leaf_width_m: f32) -> f32 {
    if wind_speed_m_s <= 0.0 || leaf_width_m <= 0.0 {
        return 0.0;
    }
    let gb = 0.147 * (wind_speed_m_s / leaf_width_m).sqrt();
    tracing::trace!(
        wind_speed_m_s,
        leaf_width_m,
        gb,
        "boundary_layer_conductance"
    );
    gb
}

/// Total leaf conductance combining stomatal and boundary layer (mol H₂O/m²/s).
///
/// Resistances in series: `1/g_total = 1/g_s + 1/g_b`
///
/// - `stomatal_conductance` — g_s (mol H₂O/m²/s)
/// - `boundary_conductance` — g_b (mol H₂O/m²/s)
#[must_use]
#[inline]
pub fn total_leaf_conductance(stomatal_conductance: f32, boundary_conductance: f32) -> f32 {
    if stomatal_conductance <= 0.0 || boundary_conductance <= 0.0 {
        return 0.0;
    }
    let total = 1.0 / (1.0 / stomatal_conductance + 1.0 / boundary_conductance);
    tracing::trace!(
        stomatal_conductance,
        boundary_conductance,
        total,
        "total_leaf_conductance"
    );
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Vapor pressure ---

    #[test]
    fn svp_at_20c() {
        let es = saturation_vapor_pressure(20.0);
        // Bolton 1980: e_s(20°C) ≈ 2.338 kPa
        assert!((es - 2.338).abs() < 0.05, "got {es}");
    }

    #[test]
    fn svp_increases_with_temp() {
        let cold = saturation_vapor_pressure(10.0);
        let warm = saturation_vapor_pressure(30.0);
        assert!(warm > cold);
    }

    #[test]
    fn vpd_basic() {
        let vpd = vapor_pressure_deficit(2.338, 1.5);
        assert!((vpd - 0.838).abs() < 0.01);
    }

    #[test]
    fn vpd_saturated_is_zero() {
        let vpd = vapor_pressure_deficit(2.0, 2.0);
        assert_eq!(vpd, 0.0);
    }

    #[test]
    fn vpd_supersaturated_clamped() {
        let vpd = vapor_pressure_deficit(2.0, 3.0);
        assert_eq!(vpd, 0.0);
    }

    // --- Ball-Berry ---

    #[test]
    fn ball_berry_basic() {
        // g0=0.02, g1=9, A=15 µmol/m²/s, RH=0.7, CO2=400 ppm
        let gs = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 400.0);
        // gs = 0.02 + 9 × (15 × 0.7 / 400) = 0.02 + 9 × 0.02625 = 0.02 + 0.236 = 0.256
        assert!((gs - 0.256).abs() < 0.01, "got {gs}");
    }

    #[test]
    fn ball_berry_no_photosynthesis() {
        let gs = ball_berry_conductance(0.02, 9.0, 0.0, 0.7, 400.0);
        assert!((gs - 0.02).abs() < 0.001, "should return g0");
    }

    #[test]
    fn ball_berry_dry_air_reduces_conductance() {
        let humid = ball_berry_conductance(0.02, 9.0, 15.0, 0.9, 400.0);
        let dry = ball_berry_conductance(0.02, 9.0, 15.0, 0.3, 400.0);
        assert!(humid > dry, "dry air should reduce conductance");
    }

    #[test]
    fn ball_berry_high_co2_reduces_conductance() {
        let low_co2 = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 280.0);
        let high_co2 = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 800.0);
        assert!(low_co2 > high_co2, "high CO2 reduces stomatal opening");
    }

    #[test]
    fn ball_berry_zero_co2() {
        let gs = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 0.0);
        assert!((gs - 0.02).abs() < 0.001, "zero CO2 should return g0");
    }

    // --- Transpiration ---

    #[test]
    fn transpiration_basic() {
        // gs=0.25 mol/m²/s, VPD=1.5 kPa, P=101.3 kPa
        let e = transpiration_rate(0.25, 1.5, 101.3);
        // E = 0.25 × 1.5 / 101.3 × 1000 ≈ 3.70 mmol/m²/s
        assert!((e - 3.70).abs() < 0.1, "got {e}");
    }

    #[test]
    fn transpiration_zero_conductance() {
        assert_eq!(transpiration_rate(0.0, 1.5, 101.3), 0.0);
    }

    #[test]
    fn transpiration_zero_vpd() {
        assert_eq!(transpiration_rate(0.25, 0.0, 101.3), 0.0);
    }

    #[test]
    fn transpiration_increases_with_vpd() {
        let low = transpiration_rate(0.25, 0.5, 101.3);
        let high = transpiration_rate(0.25, 3.0, 101.3);
        assert!(high > low);
    }

    // --- WUE ---

    #[test]
    fn wue_basic() {
        // A=15 µmol/m²/s, E=3.7 mmol/m²/s → WUE ≈ 4.05
        let wue = instantaneous_wue(15.0, 3.7);
        assert!((wue - 4.05).abs() < 0.1, "got {wue}");
    }

    #[test]
    fn wue_zero_transpiration() {
        assert_eq!(instantaneous_wue(15.0, 0.0), 0.0);
    }

    // --- Drought factor ---

    #[test]
    fn drought_factor_wet() {
        let f = drought_stomatal_factor(0.4, 0.15, 0.35, StomatalBehavior::Anisohydric);
        assert_eq!(f, 1.0, "above field capacity → fully open");
    }

    #[test]
    fn drought_factor_wilted() {
        let f = drought_stomatal_factor(0.10, 0.15, 0.35, StomatalBehavior::Anisohydric);
        assert_eq!(f, 0.0, "below wilting → fully closed");
    }

    #[test]
    fn drought_factor_mid() {
        let f = drought_stomatal_factor(0.25, 0.15, 0.35, StomatalBehavior::Anisohydric);
        assert!(
            (f - 0.5).abs() < 0.01,
            "linear midpoint for anisohydric, got {f}"
        );
    }

    #[test]
    fn isohydric_closes_earlier() {
        let aniso = drought_stomatal_factor(0.25, 0.15, 0.35, StomatalBehavior::Anisohydric);
        let iso = drought_stomatal_factor(0.25, 0.15, 0.35, StomatalBehavior::Isohydric);
        assert!(
            iso < aniso,
            "isohydric should close earlier: iso={iso}, aniso={aniso}"
        );
    }

    // --- VPD factor ---

    #[test]
    fn vpd_factor_no_deficit() {
        assert_eq!(vpd_stomatal_factor(0.0, 1.5), 1.0);
    }

    #[test]
    fn vpd_factor_at_d0() {
        let f = vpd_stomatal_factor(1.5, 1.5);
        assert!(
            (f - 0.5).abs() < 0.01,
            "at D0, conductance should halve, got {f}"
        );
    }

    #[test]
    fn vpd_factor_high_deficit() {
        let f = vpd_stomatal_factor(5.0, 1.5);
        assert!(
            f < 0.3,
            "high VPD should strongly reduce conductance, got {f}"
        );
    }

    // --- Boundary layer ---

    #[test]
    fn boundary_layer_basic() {
        // wind=2 m/s, leaf=0.05m → gb = 0.147 × sqrt(2/0.05) = 0.147 × 6.32 = 0.929
        let gb = boundary_layer_conductance(2.0, 0.05);
        assert!((gb - 0.929).abs() < 0.05, "got {gb}");
    }

    #[test]
    fn boundary_layer_zero_wind() {
        assert_eq!(boundary_layer_conductance(0.0, 0.05), 0.0);
    }

    #[test]
    fn boundary_layer_increases_with_wind() {
        let calm = boundary_layer_conductance(0.5, 0.05);
        let windy = boundary_layer_conductance(5.0, 0.05);
        assert!(windy > calm);
    }

    #[test]
    fn small_leaves_higher_boundary_conductance() {
        let small = boundary_layer_conductance(2.0, 0.02);
        let large = boundary_layer_conductance(2.0, 0.10);
        assert!(small > large, "small leaves have thinner boundary layers");
    }

    // --- Total conductance ---

    #[test]
    fn total_conductance_series() {
        // 1/g = 1/0.3 + 1/0.9 = 3.33 + 1.11 = 4.44 → g = 0.225
        let g = total_leaf_conductance(0.3, 0.9);
        assert!((g - 0.225).abs() < 0.01, "got {g}");
    }

    #[test]
    fn total_conductance_limited_by_smaller() {
        let g = total_leaf_conductance(0.1, 10.0);
        assert!(g < 0.1, "total should be less than the smaller component");
    }

    #[test]
    fn total_conductance_zero_stomatal() {
        assert_eq!(total_leaf_conductance(0.0, 0.9), 0.0);
    }

    #[test]
    fn total_conductance_zero_boundary() {
        assert_eq!(total_leaf_conductance(0.3, 0.0), 0.0);
    }
}
