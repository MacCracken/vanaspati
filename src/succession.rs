//! Succession dynamics — pioneer vs. climax species, shade tolerance,
//! and community replacement over time.
//!
//! Models the trade-off between fast growth in open conditions (pioneers)
//! and shade tolerance for persistence under canopy (climax species).

use serde::{Deserialize, Serialize};

/// Successional stage of a plant community or species strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SuccessionalStage {
    /// Early successional — colonizers of bare ground, disturbed sites.
    /// Fast growth, shade intolerant, short-lived. Examples: birch, fireweed, grasses.
    Pioneer,
    /// Mid-successional — moderate growth and shade tolerance.
    /// Examples: pine, Douglas fir, maple.
    MidSuccessional,
    /// Late successional — slow growth, shade tolerant, long-lived.
    /// Examples: beech, hemlock, sugar maple.
    Climax,
}

/// Shade tolerance class (0.0–1.0).
///
/// Determines the minimum light fraction (relative to full sun) at which
/// a species can maintain positive carbon balance and survive.
///
/// Pioneer species need high light; climax species persist in shade.
///
/// - `stage` — successional strategy
#[must_use]
#[inline]
pub fn shade_tolerance(stage: SuccessionalStage) -> f32 {
    let tolerance = match stage {
        SuccessionalStage::Pioneer => 0.15, // needs >85% full light
        SuccessionalStage::MidSuccessional => 0.50, // survives at 50% light
        SuccessionalStage::Climax => 0.80,  // survives at 20% light
    };
    tracing::trace!(?stage, tolerance, "shade_tolerance");
    tolerance
}

/// Maximum relative growth rate for a successional stage (dimensionless multiplier).
///
/// Pioneer species grow fastest in open conditions but are outcompeted
/// once canopy closes. This captures the fundamental growth-tolerance tradeoff.
///
/// Returns a multiplier on base growth rate (1.0 = reference).
///
/// - `stage` — successional strategy
#[must_use]
#[inline]
pub fn max_growth_rate_multiplier(stage: SuccessionalStage) -> f32 {
    let multiplier = match stage {
        SuccessionalStage::Pioneer => 2.0, // 2× faster than baseline
        SuccessionalStage::MidSuccessional => 1.0, // baseline
        SuccessionalStage::Climax => 0.5,  // half speed but persistent
    };
    tracing::trace!(?stage, multiplier, "max_growth_rate_multiplier");
    multiplier
}

/// Typical lifespan class for a successional stage (years).
///
/// Pioneer species are short-lived (decades), climax species are long-lived (centuries).
///
/// - `stage` — successional strategy
#[must_use]
#[inline]
pub fn typical_lifespan_years(stage: SuccessionalStage) -> f32 {
    let years = match stage {
        SuccessionalStage::Pioneer => 30.0,
        SuccessionalStage::MidSuccessional => 150.0,
        SuccessionalStage::Climax => 500.0,
    };
    tracing::trace!(?stage, years, "typical_lifespan_years");
    years
}

/// Light-dependent establishment probability (0.0–1.0).
///
/// Whether a seedling can establish depends on available light relative
/// to the species' shade tolerance.
///
/// `probability = ((light_fraction - min_light) / (1.0 - min_light))^p`
///
/// where min_light = 1.0 - shade_tolerance, and p shapes the response:
/// - Pioneer (p=0.5): rapidly reaches max establishment at moderate light
/// - Climax (p=2.0): establishes even in low light but slowly increases
///
/// - `light_fraction` — available light as fraction of full sun (0.0–1.0)
/// - `stage` — successional strategy of the species
#[must_use]
pub fn establishment_probability(light_fraction: f32, stage: SuccessionalStage) -> f32 {
    let tolerance = shade_tolerance(stage);
    let min_light = 1.0 - tolerance;
    let light = light_fraction.clamp(0.0, 1.0);

    if light < min_light {
        return 0.0;
    }

    let range = 1.0 - min_light;
    if range <= 0.0 {
        return 1.0;
    }

    let relative = (light - min_light) / range;
    let exponent = match stage {
        SuccessionalStage::Pioneer => 0.5, // fast response to light
        SuccessionalStage::MidSuccessional => 1.0, // linear
        SuccessionalStage::Climax => 2.0,  // slow but persistent
    };
    let prob = relative.powf(exponent).clamp(0.0, 1.0);
    tracing::trace!(
        light_fraction,
        ?stage,
        min_light,
        relative,
        prob,
        "establishment_probability"
    );
    prob
}

/// Competitive displacement probability (0.0–1.0).
///
/// The probability that a shade-tolerant species displaces a shade-intolerant
/// one at a given canopy light level. As canopy closes, pioneers lose vigor
/// and climax species gain advantage.
///
/// `displacement = shade_tolerance(invader) × (1.0 - light_fraction)`
///
/// High canopy closure (low light) + high shade tolerance → displacement.
///
/// - `light_fraction` — understory light as fraction of full sun (0.0–1.0)
/// - `invader_stage` — successional strategy of the incoming species
#[must_use]
#[inline]
pub fn competitive_displacement(light_fraction: f32, invader_stage: SuccessionalStage) -> f32 {
    let light = light_fraction.clamp(0.0, 1.0);
    let tolerance = shade_tolerance(invader_stage);
    let displacement = (tolerance * (1.0 - light)).clamp(0.0, 1.0);
    tracing::trace!(
        light_fraction,
        ?invader_stage,
        tolerance,
        displacement,
        "competitive_displacement"
    );
    displacement
}

/// Effective growth rate under canopy light conditions (dimensionless multiplier).
///
/// Combines the species' maximum growth potential with light availability.
/// Pioneer species grow fast in sun but decline sharply in shade.
/// Climax species grow slowly but maintain rate under low light.
///
/// `effective = max_multiplier × light_response`
///
/// Light response:
/// - Pioneer: `(light / 0.85)^2` — quadratic decline below 85% light
/// - MidSuccessional: `light^1` — linear with light
/// - Climax: `light^0.5` — diminishing returns, efficient at low light
///
/// - `light_fraction` — available light as fraction of full sun (0.0–1.0)
/// - `stage` — successional strategy
#[must_use]
pub fn effective_growth_multiplier(light_fraction: f32, stage: SuccessionalStage) -> f32 {
    let light = light_fraction.clamp(0.0, 1.0);
    let max_mult = max_growth_rate_multiplier(stage);

    let light_response = match stage {
        SuccessionalStage::Pioneer => {
            // Quadratic decline — pioneers need high light
            let scaled = (light / 0.85).min(1.0);
            scaled * scaled
        }
        SuccessionalStage::MidSuccessional => {
            // Linear with light
            light
        }
        SuccessionalStage::Climax => {
            // Square root — efficient at low light
            light.sqrt()
        }
    };

    let effective = max_mult * light_response;
    tracing::trace!(
        light_fraction,
        ?stage,
        max_mult,
        light_response,
        effective,
        "effective_growth_multiplier"
    );
    effective
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- shade tolerance ---

    #[test]
    fn climax_more_shade_tolerant() {
        assert!(
            shade_tolerance(SuccessionalStage::Climax)
                > shade_tolerance(SuccessionalStage::Pioneer)
        );
    }

    #[test]
    fn shade_tolerance_in_range() {
        for stage in [
            SuccessionalStage::Pioneer,
            SuccessionalStage::MidSuccessional,
            SuccessionalStage::Climax,
        ] {
            let t = shade_tolerance(stage);
            assert!((0.0..=1.0).contains(&t), "{stage:?}: {t}");
        }
    }

    // --- growth rate multiplier ---

    #[test]
    fn pioneer_grows_fastest() {
        assert!(
            max_growth_rate_multiplier(SuccessionalStage::Pioneer)
                > max_growth_rate_multiplier(SuccessionalStage::Climax)
        );
    }

    #[test]
    fn mid_successional_baseline() {
        assert_eq!(
            max_growth_rate_multiplier(SuccessionalStage::MidSuccessional),
            1.0
        );
    }

    // --- lifespan ---

    #[test]
    fn climax_lives_longest() {
        assert!(
            typical_lifespan_years(SuccessionalStage::Climax)
                > typical_lifespan_years(SuccessionalStage::Pioneer)
        );
    }

    #[test]
    fn lifespan_ordering() {
        let p = typical_lifespan_years(SuccessionalStage::Pioneer);
        let m = typical_lifespan_years(SuccessionalStage::MidSuccessional);
        let c = typical_lifespan_years(SuccessionalStage::Climax);
        assert!(p < m);
        assert!(m < c);
    }

    // --- establishment probability ---

    #[test]
    fn establishment_full_light_all_establish() {
        for stage in [
            SuccessionalStage::Pioneer,
            SuccessionalStage::MidSuccessional,
            SuccessionalStage::Climax,
        ] {
            let p = establishment_probability(1.0, stage);
            assert!((p - 1.0).abs() < 0.01, "{stage:?} at full light: {p}");
        }
    }

    #[test]
    fn establishment_no_light() {
        for stage in [
            SuccessionalStage::Pioneer,
            SuccessionalStage::MidSuccessional,
            SuccessionalStage::Climax,
        ] {
            let p = establishment_probability(0.0, stage);
            assert_eq!(p, 0.0, "{stage:?} at no light should be 0");
        }
    }

    #[test]
    fn pioneer_needs_more_light_to_establish() {
        let light = 0.3; // 30% full sun
        let pioneer = establishment_probability(light, SuccessionalStage::Pioneer);
        let climax = establishment_probability(light, SuccessionalStage::Climax);
        assert!(
            climax > pioneer,
            "climax should establish better at low light: climax={climax}, pioneer={pioneer}"
        );
    }

    #[test]
    fn establishment_below_minimum_is_zero() {
        // Pioneer needs >85% light
        let p = establishment_probability(0.10, SuccessionalStage::Pioneer);
        assert_eq!(p, 0.0, "pioneer can't establish at 10% light");
    }

    // --- competitive displacement ---

    #[test]
    fn displacement_full_light_is_zero() {
        let d = competitive_displacement(1.0, SuccessionalStage::Climax);
        assert_eq!(d, 0.0, "full light → no displacement");
    }

    #[test]
    fn displacement_dark_understory() {
        let d = competitive_displacement(0.1, SuccessionalStage::Climax);
        assert!(
            d > 0.5,
            "dark understory → strong climax displacement, got {d}"
        );
    }

    #[test]
    fn climax_displaces_better_than_pioneer() {
        let light = 0.3;
        let climax = competitive_displacement(light, SuccessionalStage::Climax);
        let pioneer = competitive_displacement(light, SuccessionalStage::Pioneer);
        assert!(climax > pioneer);
    }

    // --- effective growth multiplier ---

    #[test]
    fn pioneer_fastest_in_full_sun() {
        let p = effective_growth_multiplier(1.0, SuccessionalStage::Pioneer);
        let c = effective_growth_multiplier(1.0, SuccessionalStage::Climax);
        assert!(p > c, "pioneer should dominate in full sun: p={p}, c={c}");
    }

    #[test]
    fn climax_better_in_shade() {
        let light = 0.15;
        let p = effective_growth_multiplier(light, SuccessionalStage::Pioneer);
        let c = effective_growth_multiplier(light, SuccessionalStage::Climax);
        assert!(c > p, "climax should do better in shade: c={c}, p={p}");
    }

    #[test]
    fn effective_growth_zero_light() {
        for stage in [
            SuccessionalStage::Pioneer,
            SuccessionalStage::MidSuccessional,
            SuccessionalStage::Climax,
        ] {
            assert_eq!(
                effective_growth_multiplier(0.0, stage),
                0.0,
                "{stage:?} at zero light"
            );
        }
    }

    #[test]
    fn crossover_point_exists() {
        // There should be a light level where pioneer and climax rates cross
        let mut pioneer_better = false;
        let mut climax_better = false;
        for i in 0..=10 {
            let light = i as f32 / 10.0;
            let p = effective_growth_multiplier(light, SuccessionalStage::Pioneer);
            let c = effective_growth_multiplier(light, SuccessionalStage::Climax);
            if p > c {
                pioneer_better = true;
            }
            if c > p {
                climax_better = true;
            }
        }
        assert!(
            pioneer_better && climax_better,
            "should have a crossover between pioneer and climax dominance"
        );
    }
}
