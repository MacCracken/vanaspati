//! Evapotranspiration — Penman-Monteith energy-balance model.
//!
//! Computes actual evapotranspiration from net radiation, air temperature,
//! humidity, wind speed, and canopy conductance. This bridges the stomata
//! module (conductance) to the water module (soil water removal).

/// Slope of saturation vapor pressure curve (kPa/°C).
///
/// `Δ = 4098 × e_s / (T + 237.3)²`
///
/// where e_s = saturation vapor pressure (Magnus formula).
///
/// - `temp_celsius` — air temperature (°C)
#[must_use]
#[inline]
pub fn svp_slope(temp_celsius: f32) -> f32 {
    let es = 0.6108 * (17.27 * temp_celsius / (temp_celsius + 237.3)).exp();
    let denom = (temp_celsius + 237.3) * (temp_celsius + 237.3);
    let slope = 4098.0 * es / denom;
    tracing::trace!(temp_celsius, es, slope, "svp_slope");
    slope
}

/// Psychrometric constant (kPa/°C).
///
/// `γ = c_p × P / (ε × λ)`
///
/// At standard pressure (101.3 kPa): γ ≈ 0.0665 kPa/°C.
///
/// - `pressure_kpa` — atmospheric pressure (kPa)
#[must_use]
#[inline]
pub fn psychrometric_constant(pressure_kpa: f32) -> f32 {
    // c_p = 1.013e-3 MJ/kg/°C, ε = 0.622, λ = 2.45 MJ/kg
    let gamma = 0.000665 * pressure_kpa;
    tracing::trace!(pressure_kpa, gamma, "psychrometric_constant");
    gamma
}

/// Penman-Monteith evapotranspiration (mm/day).
///
/// The FAO-56 Penman-Monteith equation for actual ET:
///
/// ```text
///            Δ(Rn - G) + ρ_a × c_p × VPD / r_a
/// ET = ──────────────────────────────────────────
///            Δ + γ(1 + r_s / r_a)
/// ```
///
/// Simplified to avoid density/heat capacity constants:
///
/// ```text
///            Δ(Rn - G) + γ × (900/(T+273)) × u₂ × VPD
/// ET₀ = ─────────────────────────────────────────────────
///                  Δ + γ(1 + C_d × u₂)
/// ```
///
/// where C_d = r_s / r_a proxy.
///
/// This function uses the full resistance form for flexibility.
///
/// - `net_radiation_mj_m2_day` — net radiation (MJ/m²/day)
/// - `ground_heat_flux_mj_m2_day` — soil heat flux (MJ/m²/day, typically ~0.1 Rn)
/// - `temp_celsius` — mean daily air temperature (°C)
/// - `vpd_kpa` — vapor pressure deficit (kPa)
/// - `wind_speed_m_s` — wind speed at 2m height (m/s)
/// - `surface_resistance_s_m` — canopy surface resistance (s/m, from stomatal conductance)
/// - `pressure_kpa` — atmospheric pressure (kPa, default ~101.3)
#[must_use]
pub fn penman_monteith_et(
    net_radiation_mj_m2_day: f32,
    ground_heat_flux_mj_m2_day: f32,
    temp_celsius: f32,
    vpd_kpa: f32,
    wind_speed_m_s: f32,
    surface_resistance_s_m: f32,
    pressure_kpa: f32,
) -> f32 {
    if vpd_kpa <= 0.0 {
        return 0.0;
    }

    let delta = svp_slope(temp_celsius);
    let gamma = psychrometric_constant(pressure_kpa);

    // Aerodynamic resistance (s/m) — FAO simplified for grass reference
    // r_a = 208 / u₂ for standard conditions
    let wind = wind_speed_m_s.max(0.5); // minimum 0.5 m/s to avoid division issues
    let r_a = 208.0 / wind;

    let rn_g = (net_radiation_mj_m2_day - ground_heat_flux_mj_m2_day).max(0.0);

    // Air density × specific heat / r_a (simplified units)
    // ρ_a × c_p = 1.013 kJ/m³/°C ≈ 0.001013 MJ/m³/°C
    // Convert to daily: × 86400 s/day
    let aero_term = 86400.0 * 0.001013 * vpd_kpa / r_a;

    let numerator = delta * rn_g + gamma * aero_term;
    let denominator = delta + gamma * (1.0 + surface_resistance_s_m / r_a);

    if denominator <= 0.0 {
        return 0.0;
    }

    // Convert MJ/m²/day → mm/day (latent heat of vaporization ≈ 2.45 MJ/kg)
    let et = (numerator / denominator) / 2.45;
    let result = et.max(0.0);

    tracing::trace!(
        net_radiation_mj_m2_day,
        ground_heat_flux_mj_m2_day,
        temp_celsius,
        vpd_kpa,
        wind_speed_m_s,
        surface_resistance_s_m,
        delta,
        gamma,
        et = result,
        "penman_monteith_et"
    );
    result
}

/// Surface resistance from stomatal conductance (s/m).
///
/// Converts stomatal conductance (mol/m²/s) to surface resistance (s/m)
/// for use in Penman-Monteith.
///
/// `r_s = 1 / (g_s × 0.0224)` (at 20°C, 101.3 kPa)
///
/// 0.0224 m³/mol = molar volume of air at standard conditions.
/// Divided by LAI to scale from leaf to canopy.
///
/// - `stomatal_conductance_mol_m2_s` — stomatal conductance (mol H₂O/m²/s)
/// - `lai` — leaf area index (m²/m²)
#[must_use]
#[inline]
pub fn surface_resistance(stomatal_conductance_mol_m2_s: f32, lai: f32) -> f32 {
    if stomatal_conductance_mol_m2_s <= 0.0 || lai <= 0.0 {
        return f32::MAX; // infinite resistance = no conductance
    }
    let canopy_conductance = stomatal_conductance_mol_m2_s * lai * 0.0224;
    let r_s = 1.0 / canopy_conductance;
    tracing::trace!(
        stomatal_conductance_mol_m2_s,
        lai,
        canopy_conductance,
        r_s,
        "surface_resistance"
    );
    r_s
}

/// Reference evapotranspiration (ET₀) using FAO-56 short crop method (mm/day).
///
/// Simplified Penman-Monteith for a hypothetical grass reference surface
/// (height 0.12 m, r_s = 70 s/m, albedo 0.23).
///
/// Useful as a baseline — actual ET = ET₀ × crop coefficient.
///
/// - `net_radiation_mj_m2_day` — net radiation (MJ/m²/day)
/// - `temp_celsius` — mean daily temperature (°C)
/// - `vpd_kpa` — vapor pressure deficit (kPa)
/// - `wind_speed_m_s` — wind speed at 2m (m/s)
#[must_use]
#[inline]
pub fn reference_et(
    net_radiation_mj_m2_day: f32,
    temp_celsius: f32,
    vpd_kpa: f32,
    wind_speed_m_s: f32,
) -> f32 {
    penman_monteith_et(
        net_radiation_mj_m2_day,
        0.0, // G ≈ 0 for daily timestep (FAO-56)
        temp_celsius,
        vpd_kpa,
        wind_speed_m_s,
        70.0,  // grass reference: 70 s/m
        101.3, // standard pressure
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- svp_slope ---

    #[test]
    fn svp_slope_at_20c() {
        let s = svp_slope(20.0);
        // Expected: ~0.145 kPa/°C
        assert!((s - 0.145).abs() < 0.01, "got {s}");
    }

    #[test]
    fn svp_slope_increases_with_temp() {
        let cool = svp_slope(10.0);
        let warm = svp_slope(30.0);
        assert!(warm > cool);
    }

    // --- psychrometric constant ---

    #[test]
    fn psychrometric_at_sea_level() {
        let gamma = psychrometric_constant(101.3);
        assert!((gamma - 0.0674).abs() < 0.002, "got {gamma}");
    }

    // --- penman-monteith ---

    #[test]
    fn pm_typical_summer_day() {
        // Typical temperate summer: Rn=15 MJ/m²/day, T=25°C, VPD=1.5 kPa, u=2 m/s
        let et = penman_monteith_et(15.0, 1.5, 25.0, 1.5, 2.0, 70.0, 101.3);
        // ET₀ should be ~4-6 mm/day in summer
        assert!((2.0..=8.0).contains(&et), "typical summer ET, got {et}");
    }

    #[test]
    fn pm_zero_vpd_zero_et() {
        let et = penman_monteith_et(15.0, 0.0, 25.0, 0.0, 2.0, 70.0, 101.3);
        assert_eq!(et, 0.0);
    }

    #[test]
    fn pm_higher_vpd_more_et() {
        let low = penman_monteith_et(15.0, 0.0, 25.0, 0.5, 2.0, 70.0, 101.3);
        let high = penman_monteith_et(15.0, 0.0, 25.0, 3.0, 2.0, 70.0, 101.3);
        assert!(high > low);
    }

    #[test]
    fn pm_higher_resistance_less_et() {
        let open = penman_monteith_et(15.0, 0.0, 25.0, 1.5, 2.0, 50.0, 101.3);
        let closed = penman_monteith_et(15.0, 0.0, 25.0, 1.5, 2.0, 500.0, 101.3);
        assert!(open > closed, "closed stomata should reduce ET");
    }

    #[test]
    fn pm_more_radiation_more_et() {
        let cloudy = penman_monteith_et(5.0, 0.0, 25.0, 1.5, 2.0, 70.0, 101.3);
        let sunny = penman_monteith_et(20.0, 0.0, 25.0, 1.5, 2.0, 70.0, 101.3);
        assert!(sunny > cloudy);
    }

    // --- surface resistance ---

    #[test]
    fn surface_resistance_basic() {
        // gs = 0.25 mol/m²/s, LAI = 4
        let rs = surface_resistance(0.25, 4.0);
        // canopy_g = 0.25 × 4 × 0.0224 = 0.0224, rs = 1/0.0224 ≈ 44.6
        assert!((rs - 44.6).abs() < 1.0, "got {rs}");
    }

    #[test]
    fn surface_resistance_zero_conductance() {
        let rs = surface_resistance(0.0, 4.0);
        assert_eq!(rs, f32::MAX);
    }

    #[test]
    fn surface_resistance_zero_lai() {
        let rs = surface_resistance(0.25, 0.0);
        assert_eq!(rs, f32::MAX);
    }

    #[test]
    fn surface_resistance_decreases_with_lai() {
        let sparse = surface_resistance(0.25, 1.0);
        let dense = surface_resistance(0.25, 6.0);
        assert!(dense < sparse, "more leaves = less resistance");
    }

    // --- reference ET ---

    #[test]
    fn reference_et_summer() {
        let et0 = reference_et(15.0, 25.0, 1.5, 2.0);
        assert!((2.0..=8.0).contains(&et0), "got {et0}");
    }

    #[test]
    fn reference_et_winter_low() {
        let et0 = reference_et(3.0, 5.0, 0.3, 1.0);
        assert!(et0 < 2.0, "winter ET should be low, got {et0}");
    }
}
