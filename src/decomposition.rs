use serde::{Deserialize, Serialize};

/// Litter type — determines base decomposition rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LitterType {
    /// Leaf litter (fast decay, 1–3 year turnover).
    Leaf,
    /// Fine root litter (moderate decay).
    FineRoot,
    /// Coarse woody debris (slow decay, 5–20 year turnover).
    Wood,
    /// Reproductive material — fruit, seeds (fastest decay).
    Reproductive,
}

/// Base annual decomposition rate constant at 25°C and optimal moisture (per year).
///
/// Based on global litter decomposition meta-analyses (Zhang et al. 2008, Cornwell et al. 2008).
///
/// - Leaf: k = 1.5/yr (typical broadleaf, ~8 month half-life)
/// - FineRoot: k = 0.8/yr (~10 month half-life)
/// - Wood: k = 0.1/yr (~7 year half-life)
/// - Reproductive: k = 3.0/yr (~3 month half-life)
#[must_use]
pub fn base_decomposition_rate(litter_type: LitterType) -> f32 {
    let rate = match litter_type {
        LitterType::Leaf => 1.5,         // per year
        LitterType::FineRoot => 0.8,     // per year
        LitterType::Wood => 0.1,         // per year
        LitterType::Reproductive => 3.0, // per year
    };
    tracing::trace!(?litter_type, rate, "base_decomposition_rate");
    rate
}

/// Temperature effect on decomposition (Q10 model).
///
/// `factor = Q10^((T - T_ref) / 10)`
///
/// Q10 = 2.0 (decomposition rate doubles per 10°C rise).
/// Reference temperature T_ref = 25°C.
/// Below 0°C, returns 0.0 (frozen soil halts decomposition).
///
/// - `temp_celsius` — soil/litter temperature (°C)
#[must_use]
#[inline]
pub fn temperature_decomposition_factor(temp_celsius: f32) -> f32 {
    if temp_celsius < 0.0 {
        return 0.0;
    }
    let q10 = 2.0_f32;
    let t_ref = 25.0_f32;
    let factor = q10.powf((temp_celsius - t_ref) / 10.0);
    tracing::trace!(temp_celsius, factor, "temperature_decomposition_factor");
    factor
}

/// Moisture effect on decomposition (0.0–1.0).
///
/// Bell curve with optimum at 60% moisture (field capacity).
/// Too dry (< 20%) or waterlogged (> 90%) both inhibit decomposition.
///
/// `factor = e^(-((moisture - 0.6)² / 0.08))`
///
/// σ² = 0.08 gives a curve that drops to ~0.1 at extremes.
///
/// - `moisture_fraction` — soil moisture (0.0–1.0, where 0.6 ≈ field capacity)
#[must_use]
#[inline]
pub fn moisture_decomposition_factor(moisture_fraction: f32) -> f32 {
    let m = moisture_fraction.clamp(0.0, 1.0);
    let diff = m - 0.6;
    let factor = (-diff * diff / 0.08).exp();
    tracing::trace!(moisture_fraction, factor, "moisture_decomposition_factor");
    factor
}

/// Effective daily decomposition rate constant (per day).
///
/// Combines base rate with temperature and moisture modifiers:
/// `k_daily = (k_annual / 365) × temp_factor × moisture_factor`
///
/// - `litter_type` — type of organic matter
/// - `temp_celsius` — soil/litter temperature (°C)
/// - `moisture_fraction` — soil moisture (0.0–1.0)
#[must_use]
pub fn daily_decomposition_rate(
    litter_type: LitterType,
    temp_celsius: f32,
    moisture_fraction: f32,
) -> f32 {
    let k_annual = base_decomposition_rate(litter_type);
    let temp_f = temperature_decomposition_factor(temp_celsius);
    let moist_f = moisture_decomposition_factor(moisture_fraction);
    let k_daily = (k_annual / 365.0) * temp_f * moist_f;
    tracing::trace!(
        ?litter_type,
        k_annual,
        temp_f,
        moist_f,
        k_daily,
        "daily_decomposition_rate"
    );
    k_daily
}

/// Remaining mass after decomposition (exponential decay, kg).
///
/// `mass(t) = mass_0 × e^(-k × t)`
///
/// - `initial_mass_kg` — starting litter mass (kilograms)
/// - `rate_per_day` — daily decomposition rate constant (per day)
/// - `days` — elapsed time (days)
#[must_use]
#[inline]
pub fn remaining_mass(initial_mass_kg: f32, rate_per_day: f32, days: f32) -> f32 {
    if initial_mass_kg <= 0.0 || days <= 0.0 {
        return initial_mass_kg.max(0.0);
    }
    let mass = initial_mass_kg * (-rate_per_day * days).exp();
    tracing::trace!(initial_mass_kg, rate_per_day, days, mass, "remaining_mass");
    mass
}

/// Mass lost to decomposition over a period (kg).
///
/// `lost = mass_0 - mass_0 × e^(-k × t)`
///
/// - `initial_mass_kg` — starting litter mass (kilograms)
/// - `rate_per_day` — daily decomposition rate constant (per day)
/// - `days` — elapsed time (days)
#[must_use]
#[inline]
pub fn mass_decomposed(initial_mass_kg: f32, rate_per_day: f32, days: f32) -> f32 {
    if initial_mass_kg <= 0.0 || days <= 0.0 {
        return 0.0;
    }
    initial_mass_kg - remaining_mass(initial_mass_kg, rate_per_day, days)
}

/// Nitrogen released from decomposed litter (kg N).
///
/// `N_released = decomposed_mass / C:N_ratio × carbon_fraction`
///
/// Assumes ~45% of litter mass is carbon (standard for plant tissue).
/// C:N ratio determines how much nitrogen is locked in the organic matter.
///
/// Typical C:N ratios:
/// - Leaf litter: 30–60 (broadleaf ~40, conifer ~60)
/// - Fine roots: 40–60
/// - Wood: 200–500
/// - Reproductive: 15–30
///
/// - `decomposed_mass_kg` — mass lost to decomposition (kilograms)
/// - `carbon_nitrogen_ratio` — C:N ratio of the litter (dimensionless)
#[must_use]
pub fn nitrogen_release(decomposed_mass_kg: f32, carbon_nitrogen_ratio: f32) -> f32 {
    if decomposed_mass_kg <= 0.0 || carbon_nitrogen_ratio <= 0.0 {
        return 0.0;
    }
    let carbon_fraction = 0.45; // 45% of dry mass is carbon
    let n_released = decomposed_mass_kg * carbon_fraction / carbon_nitrogen_ratio;
    tracing::trace!(
        decomposed_mass_kg,
        carbon_nitrogen_ratio,
        n_released,
        "nitrogen_release"
    );
    n_released
}

/// Half-life of litter under given conditions (days).
///
/// `t_half = ln(2) / k_daily`
///
/// - `rate_per_day` — daily decomposition rate constant (per day)
#[must_use]
#[inline]
pub fn half_life_days(rate_per_day: f32) -> f32 {
    if rate_per_day <= 0.0 {
        return f32::INFINITY;
    }
    let t = 2.0_f32.ln() / rate_per_day;
    tracing::trace!(rate_per_day, t, "half_life_days");
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Base rates ---

    #[test]
    fn leaf_decomposes_faster_than_wood() {
        assert!(
            base_decomposition_rate(LitterType::Leaf) > base_decomposition_rate(LitterType::Wood)
        );
    }

    #[test]
    fn reproductive_fastest() {
        let repro = base_decomposition_rate(LitterType::Reproductive);
        assert!(repro > base_decomposition_rate(LitterType::Leaf));
        assert!(repro > base_decomposition_rate(LitterType::FineRoot));
        assert!(repro > base_decomposition_rate(LitterType::Wood));
    }

    #[test]
    fn all_base_rates_positive() {
        for lt in [
            LitterType::Leaf,
            LitterType::FineRoot,
            LitterType::Wood,
            LitterType::Reproductive,
        ] {
            assert!(base_decomposition_rate(lt) > 0.0);
        }
    }

    // --- Temperature factor ---

    #[test]
    fn temp_factor_at_reference() {
        let f = temperature_decomposition_factor(25.0);
        assert!(
            (f - 1.0).abs() < 0.01,
            "at 25°C factor should be 1.0, got {f}"
        );
    }

    #[test]
    fn temp_factor_doubles_at_35() {
        let f = temperature_decomposition_factor(35.0);
        assert!((f - 2.0).abs() < 0.01, "Q10=2 → factor=2 at 35°C, got {f}");
    }

    #[test]
    fn temp_factor_halves_at_15() {
        let f = temperature_decomposition_factor(15.0);
        assert!(
            (f - 0.5).abs() < 0.01,
            "Q10=2 → factor=0.5 at 15°C, got {f}"
        );
    }

    #[test]
    fn temp_factor_frozen() {
        assert_eq!(temperature_decomposition_factor(-5.0), 0.0);
    }

    #[test]
    fn temp_factor_increases_with_temperature() {
        let cold = temperature_decomposition_factor(10.0);
        let warm = temperature_decomposition_factor(30.0);
        assert!(warm > cold);
    }

    // --- Moisture factor ---

    #[test]
    fn moisture_optimal_near_one() {
        let f = moisture_decomposition_factor(0.6);
        assert!(
            (f - 1.0).abs() < 0.01,
            "at field capacity, factor should be ~1.0, got {f}"
        );
    }

    #[test]
    fn moisture_dry_low() {
        let f = moisture_decomposition_factor(0.1);
        assert!(f < 0.3, "very dry should inhibit decomposition, got {f}");
    }

    #[test]
    fn moisture_waterlogged_low() {
        let f = moisture_decomposition_factor(0.95);
        assert!(f < 0.3, "waterlogged should inhibit decomposition, got {f}");
    }

    #[test]
    fn moisture_clamps_input() {
        let below = moisture_decomposition_factor(-0.5);
        let at_zero = moisture_decomposition_factor(0.0);
        assert_eq!(below, at_zero, "negative should clamp to 0");

        let above = moisture_decomposition_factor(1.5);
        let at_one = moisture_decomposition_factor(1.0);
        assert_eq!(above, at_one, ">1 should clamp to 1");
    }

    #[test]
    fn moisture_symmetric_around_optimum() {
        let dry = moisture_decomposition_factor(0.4); // 0.2 below optimum
        let wet = moisture_decomposition_factor(0.8); // 0.2 above optimum
        assert!((dry - wet).abs() < 0.01, "should be symmetric");
    }

    // --- Daily rate ---

    #[test]
    fn daily_rate_leaf_warm_moist() {
        let k = daily_decomposition_rate(LitterType::Leaf, 25.0, 0.6);
        // k_annual=1.5, temp=1.0, moist=1.0 → k_daily = 1.5/365 ≈ 0.00411
        assert!((k - 0.00411).abs() < 0.001, "got {k}");
    }

    #[test]
    fn daily_rate_frozen_is_zero() {
        let k = daily_decomposition_rate(LitterType::Leaf, -5.0, 0.6);
        assert_eq!(k, 0.0, "frozen → no decomposition");
    }

    #[test]
    fn daily_rate_wood_slower_than_leaf() {
        let leaf = daily_decomposition_rate(LitterType::Leaf, 20.0, 0.5);
        let wood = daily_decomposition_rate(LitterType::Wood, 20.0, 0.5);
        assert!(leaf > wood);
    }

    // --- Remaining mass ---

    #[test]
    fn remaining_mass_decreases() {
        let early = remaining_mass(100.0, 0.01, 30.0);
        let late = remaining_mass(100.0, 0.01, 365.0);
        assert!(early > late);
        assert!(late > 0.0);
    }

    #[test]
    fn remaining_mass_zero_days() {
        assert_eq!(remaining_mass(100.0, 0.01, 0.0), 100.0);
    }

    #[test]
    fn remaining_mass_zero_initial() {
        assert_eq!(remaining_mass(0.0, 0.01, 365.0), 0.0);
    }

    #[test]
    fn remaining_mass_known_value() {
        // 100 × e^(-0.01 × 100) = 100 × e^(-1) ≈ 36.79
        let m = remaining_mass(100.0, 0.01, 100.0);
        assert!((m - 36.79).abs() < 0.5, "got {m}");
    }

    // --- Mass decomposed ---

    #[test]
    fn mass_decomposed_complement() {
        let initial = 100.0;
        let remaining = remaining_mass(initial, 0.01, 100.0);
        let lost = mass_decomposed(initial, 0.01, 100.0);
        assert!((remaining + lost - initial).abs() < 0.01);
    }

    #[test]
    fn mass_decomposed_zero_days() {
        assert_eq!(mass_decomposed(100.0, 0.01, 0.0), 0.0);
    }

    #[test]
    fn mass_decomposed_zero_mass() {
        assert_eq!(mass_decomposed(0.0, 0.01, 100.0), 0.0);
    }

    // --- Nitrogen release ---

    #[test]
    fn nitrogen_release_basic() {
        // 10 kg decomposed, C:N = 45 → N = 10 × 0.45 / 45 = 0.1 kg
        let n = nitrogen_release(10.0, 45.0);
        assert!((n - 0.1).abs() < 0.001, "got {n}");
    }

    #[test]
    fn nitrogen_release_zero_mass() {
        assert_eq!(nitrogen_release(0.0, 40.0), 0.0);
    }

    #[test]
    fn nitrogen_release_zero_cn() {
        assert_eq!(nitrogen_release(10.0, 0.0), 0.0);
    }

    #[test]
    fn nitrogen_release_high_cn_less_nitrogen() {
        let low_cn = nitrogen_release(10.0, 30.0); // leaf-like
        let high_cn = nitrogen_release(10.0, 300.0); // wood-like
        assert!(low_cn > high_cn, "low C:N should release more nitrogen");
    }

    // --- Half-life ---

    #[test]
    fn half_life_known_value() {
        // k=0.01/day → t_half = ln(2)/0.01 ≈ 69.3 days
        let t = half_life_days(0.01);
        assert!((t - 69.3).abs() < 0.5, "got {t}");
    }

    #[test]
    fn half_life_zero_rate() {
        assert_eq!(half_life_days(0.0), f32::INFINITY);
    }

    #[test]
    fn half_life_negative_rate() {
        assert_eq!(half_life_days(-0.01), f32::INFINITY);
    }

    #[test]
    fn half_life_fast_rate_short() {
        let fast = half_life_days(0.1);
        let slow = half_life_days(0.001);
        assert!(fast < slow);
    }

    #[test]
    fn half_life_matches_remaining_mass() {
        let k = 0.005;
        let t_half = half_life_days(k);
        let remaining = remaining_mass(100.0, k, t_half);
        assert!(
            (remaining - 50.0).abs() < 0.5,
            "at half-life, mass should be ~50%, got {remaining}"
        );
    }
}
