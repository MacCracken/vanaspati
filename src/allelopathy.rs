//! Allelopathy — chemical competition between plants.
//!
//! Some plants release allelochemicals (from roots, leaves, or litter) that
//! inhibit the germination or growth of neighboring plants. Classic examples:
//! black walnut (juglone), eucalyptus, garlic mustard.

use serde::{Deserialize, Serialize};

/// Allelopathic potency class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AllelopathicPotency {
    /// No allelopathic effect.
    None,
    /// Mild — slight growth reduction in sensitive neighbors.
    /// Examples: many grasses, clover.
    Mild,
    /// Moderate — noticeable growth suppression.
    /// Examples: sunflower, sorghum, pine (terpenes).
    Moderate,
    /// Strong — severe inhibition of germination and growth.
    /// Examples: black walnut (juglone), eucalyptus, ailanthus.
    Strong,
}

/// Base allelochemical production rate (arbitrary units per kg biomass per day).
///
/// Higher values mean more toxin released into the soil per unit of plant biomass.
///
/// - `potency` — allelopathic potency class
#[must_use]
#[inline]
pub fn production_rate(potency: AllelopathicPotency) -> f32 {
    let rate = match potency {
        AllelopathicPotency::None => 0.0,
        AllelopathicPotency::Mild => 0.001,
        AllelopathicPotency::Moderate => 0.005,
        AllelopathicPotency::Strong => 0.015,
    };
    tracing::trace!(?potency, rate, "production_rate");
    rate
}

/// Soil allelochemical concentration after accumulation and decay.
///
/// Allelochemicals accumulate from production and decay via microbial breakdown.
///
/// `new_conc = old_conc × decay_factor + daily_input`
///
/// Decay follows first-order kinetics with half-life ~30 days in warm moist soil,
/// slower in cold/dry conditions.
///
/// `decay_factor = e^(-k × temp_factor × moisture_factor)`
///
/// - `current_concentration` — current soil concentration (arbitrary units/m²)
/// - `daily_input` — daily allelochemical input (arbitrary units/m²)
/// - `temp_celsius` — soil temperature (°C)
/// - `moisture_fraction` — soil moisture (0.0–1.0)
#[must_use]
pub fn soil_concentration(
    current_concentration: f32,
    daily_input: f32,
    temp_celsius: f32,
    moisture_fraction: f32,
) -> f32 {
    // Decay rate: base k = ln(2)/30 ≈ 0.023/day at 25°C, optimal moisture
    let k_base = 0.023_f32;

    // Temperature factor (Q10=2, ref=25°C, frozen=0)
    let temp_f = if temp_celsius < 0.0 {
        0.0
    } else {
        2.0_f32.powf((temp_celsius - 25.0) / 10.0)
    };

    // Moisture factor — decomposition optimal at 0.6
    let m = moisture_fraction.clamp(0.0, 1.0);
    let moist_f = (-(m - 0.6) * (m - 0.6) / 0.08).exp();

    let decay_factor = (-k_base * temp_f * moist_f).exp();
    let new_conc = (current_concentration.max(0.0) * decay_factor + daily_input.max(0.0)).max(0.0);

    tracing::trace!(
        current_concentration,
        daily_input,
        temp_celsius,
        moisture_fraction,
        decay_factor,
        new_conc,
        "soil_concentration"
    );
    new_conc
}

/// Growth inhibition factor from allelochemicals (0.0–1.0).
///
/// Dose-response curve: `inhibition = 1 - e^(-sensitivity × concentration)`
///
/// Returns a *reduction* factor: 0.0 = no inhibition, 1.0 = complete inhibition.
/// Apply as: `effective_growth = base_growth × (1.0 - inhibition)`.
///
/// Sensitivity varies by target species:
/// - Tolerant species: 0.5–2.0
/// - Sensitive species: 5.0–15.0
///
/// - `concentration` — soil allelochemical concentration (arbitrary units/m²)
/// - `sensitivity` — target species sensitivity (higher = more sensitive)
#[must_use]
#[inline]
pub fn growth_inhibition(concentration: f32, sensitivity: f32) -> f32 {
    if concentration <= 0.0 || sensitivity <= 0.0 {
        return 0.0;
    }
    let inhibition = (1.0 - (-sensitivity * concentration).exp()).clamp(0.0, 1.0);
    tracing::trace!(concentration, sensitivity, inhibition, "growth_inhibition");
    inhibition
}

/// Germination inhibition factor (0.0–1.0).
///
/// Allelochemicals typically inhibit germination more strongly than growth.
/// Uses the same dose-response but with 2× sensitivity.
///
/// `inhibition = 1 - e^(-2 × sensitivity × concentration)`
///
/// - `concentration` — soil allelochemical concentration (arbitrary units/m²)
/// - `sensitivity` — target species sensitivity
#[must_use]
#[inline]
pub fn germination_inhibition(concentration: f32, sensitivity: f32) -> f32 {
    if concentration <= 0.0 || sensitivity <= 0.0 {
        return 0.0;
    }
    let inhibition = (1.0 - (-2.0 * sensitivity * concentration).exp()).clamp(0.0, 1.0);
    tracing::trace!(
        concentration,
        sensitivity,
        inhibition,
        "germination_inhibition"
    );
    inhibition
}

/// Daily allelochemical input from a plant (arbitrary units/m²/day).
///
/// `input = biomass_kg × production_rate`
///
/// Larger plants produce more allelochemicals.
///
/// - `biomass_kg` — total above-ground biomass (kg)
/// - `potency` — allelopathic potency class
#[must_use]
#[inline]
pub fn daily_input(biomass_kg: f32, potency: AllelopathicPotency) -> f32 {
    if biomass_kg <= 0.0 {
        return 0.0;
    }
    let rate = production_rate(potency);
    let input = biomass_kg * rate;
    tracing::trace!(biomass_kg, ?potency, rate, input, "daily_input");
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- production rate ---

    #[test]
    fn none_produces_zero() {
        assert_eq!(production_rate(AllelopathicPotency::None), 0.0);
    }

    #[test]
    fn strong_produces_most() {
        assert!(
            production_rate(AllelopathicPotency::Strong)
                > production_rate(AllelopathicPotency::Moderate)
        );
    }

    #[test]
    fn ordering() {
        let n = production_rate(AllelopathicPotency::None);
        let mi = production_rate(AllelopathicPotency::Mild);
        let mo = production_rate(AllelopathicPotency::Moderate);
        let s = production_rate(AllelopathicPotency::Strong);
        assert!(n < mi);
        assert!(mi < mo);
        assert!(mo < s);
    }

    // --- soil concentration ---

    #[test]
    fn concentration_accumulates() {
        let c1 = soil_concentration(0.0, 0.01, 25.0, 0.6);
        assert!((c1 - 0.01).abs() < 0.001);
        let c2 = soil_concentration(c1, 0.01, 25.0, 0.6);
        assert!(c2 > c1, "should accumulate");
    }

    #[test]
    fn concentration_decays_without_input() {
        let c = soil_concentration(1.0, 0.0, 25.0, 0.6);
        assert!(c < 1.0, "should decay without input");
    }

    #[test]
    fn concentration_decays_faster_warm() {
        let warm = soil_concentration(1.0, 0.0, 30.0, 0.6);
        let cool = soil_concentration(1.0, 0.0, 10.0, 0.6);
        assert!(warm < cool, "warm soil degrades faster");
    }

    #[test]
    fn concentration_frozen_no_decay() {
        let c = soil_concentration(1.0, 0.0, -5.0, 0.6);
        assert!((c - 1.0).abs() < 0.01, "frozen soil preserves chemicals");
    }

    #[test]
    fn concentration_never_negative() {
        let c = soil_concentration(0.001, 0.0, 40.0, 0.6);
        assert!(c >= 0.0);
    }

    // --- growth inhibition ---

    #[test]
    fn inhibition_zero_concentration() {
        assert_eq!(growth_inhibition(0.0, 10.0), 0.0);
    }

    #[test]
    fn inhibition_zero_sensitivity() {
        assert_eq!(growth_inhibition(1.0, 0.0), 0.0);
    }

    #[test]
    fn inhibition_increases_with_concentration() {
        let low = growth_inhibition(0.1, 5.0);
        let high = growth_inhibition(1.0, 5.0);
        assert!(high > low);
    }

    #[test]
    fn inhibition_increases_with_sensitivity() {
        let tolerant = growth_inhibition(0.5, 1.0);
        let sensitive = growth_inhibition(0.5, 10.0);
        assert!(sensitive > tolerant);
    }

    #[test]
    fn inhibition_saturates_at_one() {
        let i = growth_inhibition(100.0, 10.0);
        assert!((i - 1.0).abs() < 0.01, "high dose should saturate");
    }

    #[test]
    fn inhibition_in_valid_range() {
        let i = growth_inhibition(0.5, 5.0);
        assert!((0.0..=1.0).contains(&i), "got {i}");
    }

    // --- germination inhibition ---

    #[test]
    fn germination_more_sensitive_than_growth() {
        let growth = growth_inhibition(0.3, 5.0);
        let germ = germination_inhibition(0.3, 5.0);
        assert!(germ > growth, "germination should be more inhibited");
    }

    #[test]
    fn germination_zero_at_zero() {
        assert_eq!(germination_inhibition(0.0, 5.0), 0.0);
    }

    // --- daily input ---

    #[test]
    fn daily_input_proportional_to_biomass() {
        let small = daily_input(10.0, AllelopathicPotency::Strong);
        let large = daily_input(100.0, AllelopathicPotency::Strong);
        assert!((large / small - 10.0).abs() < 0.01);
    }

    #[test]
    fn daily_input_zero_biomass() {
        assert_eq!(daily_input(0.0, AllelopathicPotency::Strong), 0.0);
    }

    #[test]
    fn daily_input_none_potency() {
        assert_eq!(daily_input(100.0, AllelopathicPotency::None), 0.0);
    }

    #[test]
    fn daily_input_strong_more_than_mild() {
        let mild = daily_input(100.0, AllelopathicPotency::Mild);
        let strong = daily_input(100.0, AllelopathicPotency::Strong);
        assert!(strong > mild);
    }
}
