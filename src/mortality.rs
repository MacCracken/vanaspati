use serde::{Deserialize, Serialize};

/// Cause of plant mortality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MortalityCause {
    Age,
    Drought,
    Competition,
    Disease,
    Frost,
}

/// Daily probability of death from aging (Weibull hazard function).
///
/// `h(t) = (k/λ) × (t/λ)^(k-1)`
///
/// k = 3.0 (increasing hazard — mortality accelerates with age),
/// λ = max_lifespan × 0.8 (scale parameter).
/// Clamped to `[0.0, 1.0]`.
///
/// - `current_age_days` — current age (days)
/// - `max_lifespan_days` — species maximum lifespan (days)
#[must_use]
pub fn age_mortality_rate(current_age_days: f32, max_lifespan_days: f32) -> f32 {
    if current_age_days <= 0.0 || max_lifespan_days <= 0.0 {
        return 0.0;
    }
    let k = 3.0_f32;
    let lambda = max_lifespan_days * 0.8;
    let ratio = current_age_days / lambda;
    let rate = (k / lambda) * ratio.powf(k - 1.0);
    let clamped = rate.clamp(0.0, 1.0);
    tracing::trace!(
        current_age_days,
        max_lifespan_days,
        rate,
        clamped,
        "age_mortality_rate"
    );
    clamped
}

/// Fraction of population that should die from self-thinning (Yoda's -3/2 power law).
///
/// `max_density = (C / mean_mass)^(2/3)`
///
/// If current density exceeds max_density:
/// `fraction = 1.0 - max_density / density`
///
/// C = 4.0 (typical for temperate forests, kg and individuals/m²).
///
/// - `density` — current population density (individuals per m²)
/// - `mean_mass_kg` — mean individual biomass (kilograms)
#[must_use]
pub fn self_thinning_mortality(density: f32, mean_mass_kg: f32) -> f32 {
    if density <= 0.0 || mean_mass_kg <= 0.0 {
        return 0.0;
    }
    let c = 4.0_f32;
    let max_density = (c / mean_mass_kg).powf(2.0 / 3.0);
    if density <= max_density {
        tracing::trace!(
            density,
            mean_mass_kg,
            max_density,
            fraction = 0.0,
            "self_thinning_mortality"
        );
        return 0.0;
    }
    let fraction = 1.0 - max_density / density;
    tracing::trace!(
        density,
        mean_mass_kg,
        max_density,
        fraction,
        "self_thinning_mortality"
    );
    fraction
}

/// Probability of frost death based on temperature vs cold hardiness.
///
/// `p = 1 / (1 + e^(k × (T - T_h)))`
///
/// Logistic curve with k = 4.0: ~1.0 when temp is well below hardiness,
/// ~0.5 at exactly hardiness threshold, ~0.0 when temp is above.
///
/// - `temp_celsius` — current temperature (degrees Celsius)
/// - `hardiness_celsius` — cold hardiness threshold (degrees Celsius, typically negative)
#[must_use]
pub fn frost_mortality(temp_celsius: f32, hardiness_celsius: f32) -> f32 {
    let k = 4.0_f32;
    let prob = 1.0 / (1.0 + (k * (temp_celsius - hardiness_celsius)).exp());
    tracing::trace!(temp_celsius, hardiness_celsius, prob, "frost_mortality");
    prob
}

/// Probability of drought death based on water deficit.
///
/// `p = max(0, 1 - available/needed)²`
///
/// Quadratic: no mortality when water is sufficient, accelerating mortality
/// as deficit increases.
///
/// - `water_available` — water available to roots (unitless ratio OK, or mm/liters)
/// - `water_needed` — water required for survival (same units as available)
#[must_use]
pub fn drought_mortality(water_available: f32, water_needed: f32) -> f32 {
    if water_needed <= 0.0 {
        return 0.0;
    }
    if water_available <= 0.0 {
        tracing::trace!(
            water_available,
            water_needed,
            prob = 1.0,
            "drought_mortality"
        );
        return 1.0;
    }
    let deficit = (1.0 - water_available / water_needed).max(0.0);
    let prob = deficit * deficit;
    tracing::trace!(
        water_available,
        water_needed,
        deficit,
        prob,
        "drought_mortality"
    );
    prob
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_mortality_increases_with_age() {
        let young = age_mortality_rate(100.0, 36500.0);
        let old = age_mortality_rate(30000.0, 36500.0);
        assert!(old > young);
    }

    #[test]
    fn age_mortality_zero_at_birth() {
        assert_eq!(age_mortality_rate(0.0, 36500.0), 0.0);
    }

    #[test]
    fn age_mortality_high_near_max() {
        let rate = age_mortality_rate(36000.0, 36500.0);
        assert!(
            rate > 0.0,
            "near max lifespan should have significant mortality"
        );
    }

    #[test]
    fn age_mortality_zero_max_lifespan() {
        assert_eq!(age_mortality_rate(100.0, 0.0), 0.0);
    }

    #[test]
    fn age_mortality_negative_age() {
        assert_eq!(age_mortality_rate(-10.0, 36500.0), 0.0);
    }

    #[test]
    fn self_thinning_zero_when_sparse() {
        // Very sparse population, large individuals
        let f = self_thinning_mortality(0.001, 100.0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn self_thinning_increases_with_density() {
        let low = self_thinning_mortality(1.0, 1.0);
        let high = self_thinning_mortality(10.0, 1.0);
        assert!(high >= low);
    }

    #[test]
    fn self_thinning_zero_density() {
        assert_eq!(self_thinning_mortality(0.0, 1.0), 0.0);
    }

    #[test]
    fn self_thinning_zero_mass() {
        assert_eq!(self_thinning_mortality(10.0, 0.0), 0.0);
    }

    #[test]
    fn frost_high_below_hardiness() {
        let p = frost_mortality(-30.0, -10.0);
        assert!(p > 0.99, "well below hardiness → near-certain death");
    }

    #[test]
    fn frost_low_above_hardiness() {
        let p = frost_mortality(10.0, -10.0);
        assert!(p < 0.01, "well above hardiness → near-zero risk");
    }

    #[test]
    fn frost_about_half_at_threshold() {
        let p = frost_mortality(-10.0, -10.0);
        assert!(
            (p - 0.5).abs() < 0.01,
            "at hardiness threshold, probability should be ~0.5, got {p}"
        );
    }

    #[test]
    fn drought_zero_when_sufficient() {
        let p = drought_mortality(200.0, 100.0);
        assert_eq!(p, 0.0, "more water than needed → no drought mortality");
    }

    #[test]
    fn drought_one_when_no_water() {
        assert_eq!(drought_mortality(0.0, 100.0), 1.0);
    }

    #[test]
    fn drought_zero_needed() {
        assert_eq!(drought_mortality(50.0, 0.0), 0.0);
    }

    #[test]
    fn drought_partial_deficit() {
        let p = drought_mortality(50.0, 100.0);
        assert!(p > 0.0 && p < 1.0);
        // deficit = 0.5, p = 0.25
        assert!((p - 0.25).abs() < 0.01);
    }

    #[test]
    fn drought_negative_available() {
        assert_eq!(drought_mortality(-10.0, 100.0), 1.0);
    }
}
