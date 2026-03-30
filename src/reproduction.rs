//! Vegetative reproduction — clonal spread via runners, rhizomes,
//! root sprouting, and layering.
//!
//! Models asexual reproduction where offspring (ramets) are produced
//! from the parent plant without seeds. Common in grasses, ferns,
//! and many forest understory species.

use serde::{Deserialize, Serialize};

/// Vegetative reproduction method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VegetativeMethod {
    /// Runners/stolons — above-ground horizontal stems (strawberry, spider plant).
    Runner,
    /// Rhizomes — below-ground horizontal stems (bamboo, iris, ferns).
    Rhizome,
    /// Root sprouting — new shoots from lateral roots (aspen, sumac).
    RootSprout,
    /// Layering — branches touching ground root and form new plants (blackberry).
    Layering,
}

/// Maximum spread distance per growth season (meters).
///
/// How far a ramet can establish from the parent per year.
///
/// - `method` — vegetative reproduction method
#[must_use]
#[inline]
pub fn spread_distance_m(method: VegetativeMethod) -> f32 {
    let dist = match method {
        VegetativeMethod::Runner => 1.5,     // stolons reach ~1.5m
        VegetativeMethod::Rhizome => 3.0,    // rhizomes can be aggressive
        VegetativeMethod::RootSprout => 5.0, // lateral roots spread far
        VegetativeMethod::Layering => 2.0,   // limited by branch reach
    };
    tracing::trace!(?method, dist, "spread_distance_m");
    dist
}

/// Ramet production rate (ramets per parent per year).
///
/// How many new plants a parent can produce vegetatively per growing season.
/// Modulated by available resources (see `resource_limited_ramets`).
///
/// - `method` — vegetative reproduction method
#[must_use]
#[inline]
pub fn base_ramet_rate(method: VegetativeMethod) -> f32 {
    let rate = match method {
        VegetativeMethod::Runner => 8.0,     // strawberry: 8–12 runners/year
        VegetativeMethod::Rhizome => 15.0,   // bamboo/grass: many shoots
        VegetativeMethod::RootSprout => 5.0, // fewer but larger offspring
        VegetativeMethod::Layering => 2.0,   // limited contact points
    };
    tracing::trace!(?method, rate, "base_ramet_rate");
    rate
}

/// Resource cost of producing one ramet (fraction of parent biomass, 0.0–1.0).
///
/// Each ramet requires carbon/nitrogen investment from the parent.
/// Higher cost → fewer ramets sustainable under stress.
///
/// - `method` — vegetative reproduction method
#[must_use]
#[inline]
pub fn ramet_cost_fraction(method: VegetativeMethod) -> f32 {
    let cost = match method {
        VegetativeMethod::Runner => 0.02, // small: just a stolon + plantlet
        VegetativeMethod::Rhizome => 0.015, // small per ramet (many produced)
        VegetativeMethod::RootSprout => 0.05, // larger: established root system
        VegetativeMethod::Layering => 0.03, // moderate: rooted branch
    };
    tracing::trace!(?method, cost, "ramet_cost_fraction");
    cost
}

/// Resource-limited ramet production (ramets/year).
///
/// Actual ramet production scales with available resources:
/// `actual = base_rate × resource_factor`
///
/// Resource factor combines water and nitrogen stress:
/// `resource_factor = min(water_stress, n_stress)`
///
/// Under severe stress, plants reduce clonal investment.
///
/// - `method` — vegetative reproduction method
/// - `water_stress_factor` — water availability (0.0–1.0)
/// - `nitrogen_stress_factor` — N availability (0.0–1.0)
#[must_use]
pub fn resource_limited_ramets(
    method: VegetativeMethod,
    water_stress_factor: f32,
    nitrogen_stress_factor: f32,
) -> f32 {
    let base = base_ramet_rate(method);
    let resource_f = water_stress_factor
        .clamp(0.0, 1.0)
        .min(nitrogen_stress_factor.clamp(0.0, 1.0));
    let actual = base * resource_f;
    tracing::trace!(
        ?method,
        base,
        water_stress_factor,
        nitrogen_stress_factor,
        resource_f,
        actual,
        "resource_limited_ramets"
    );
    actual
}

/// Clonal spread area after n years (m²).
///
/// Estimates the area colonized by vegetative spread assuming
/// radial expansion at the method's spread rate.
///
/// `area = π × (distance × years)²`
///
/// This is the potential area; actual colonization depends on
/// establishment success and competition.
///
/// - `method` — vegetative reproduction method
/// - `years` — number of growing seasons
#[must_use]
pub fn clonal_area_m2(method: VegetativeMethod, years: f32) -> f32 {
    if years <= 0.0 {
        return 0.0;
    }
    let radius = spread_distance_m(method) * years;
    let area = std::f32::consts::PI * radius * radius;
    tracing::trace!(?method, years, radius, area, "clonal_area_m2");
    area
}

/// Parent biomass cost for clonal reproduction (kg).
///
/// Total biomass the parent must invest for a given number of ramets.
///
/// `cost = parent_biomass_kg × ramet_cost_fraction × num_ramets`
///
/// - `parent_biomass_kg` — parent total biomass (kg)
/// - `method` — vegetative reproduction method
/// - `num_ramets` — number of ramets to produce
#[must_use]
pub fn parent_cost_kg(parent_biomass_kg: f32, method: VegetativeMethod, num_ramets: f32) -> f32 {
    if parent_biomass_kg <= 0.0 || num_ramets <= 0.0 {
        return 0.0;
    }
    let cost_per = ramet_cost_fraction(method);
    let total = parent_biomass_kg * cost_per * num_ramets;
    // Can't cost more than the parent has
    let capped = total.min(parent_biomass_kg);
    tracing::trace!(
        parent_biomass_kg,
        ?method,
        num_ramets,
        cost_per,
        total = capped,
        "parent_cost_kg"
    );
    capped
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- spread distance ---

    #[test]
    fn root_sprout_spreads_farthest() {
        assert!(
            spread_distance_m(VegetativeMethod::RootSprout)
                > spread_distance_m(VegetativeMethod::Runner)
        );
    }

    #[test]
    fn all_distances_positive() {
        for m in [
            VegetativeMethod::Runner,
            VegetativeMethod::Rhizome,
            VegetativeMethod::RootSprout,
            VegetativeMethod::Layering,
        ] {
            assert!(spread_distance_m(m) > 0.0, "{m:?}");
        }
    }

    // --- base ramet rate ---

    #[test]
    fn rhizome_produces_most() {
        assert!(
            base_ramet_rate(VegetativeMethod::Rhizome)
                > base_ramet_rate(VegetativeMethod::Layering)
        );
    }

    #[test]
    fn layering_produces_least() {
        for m in [
            VegetativeMethod::Runner,
            VegetativeMethod::Rhizome,
            VegetativeMethod::RootSprout,
        ] {
            assert!(
                base_ramet_rate(m) > base_ramet_rate(VegetativeMethod::Layering),
                "{m:?}"
            );
        }
    }

    // --- ramet cost ---

    #[test]
    fn root_sprout_most_expensive() {
        assert!(
            ramet_cost_fraction(VegetativeMethod::RootSprout)
                > ramet_cost_fraction(VegetativeMethod::Runner)
        );
    }

    #[test]
    fn costs_in_valid_range() {
        for m in [
            VegetativeMethod::Runner,
            VegetativeMethod::Rhizome,
            VegetativeMethod::RootSprout,
            VegetativeMethod::Layering,
        ] {
            let c = ramet_cost_fraction(m);
            assert!((0.0..=1.0).contains(&c), "{m:?}: {c}");
        }
    }

    // --- resource limited ---

    #[test]
    fn resource_limited_full_resources() {
        let actual = resource_limited_ramets(VegetativeMethod::Runner, 1.0, 1.0);
        let base = base_ramet_rate(VegetativeMethod::Runner);
        assert!((actual - base).abs() < 0.01);
    }

    #[test]
    fn resource_limited_no_water() {
        assert_eq!(
            resource_limited_ramets(VegetativeMethod::Runner, 0.0, 1.0),
            0.0
        );
    }

    #[test]
    fn resource_limited_no_nitrogen() {
        assert_eq!(
            resource_limited_ramets(VegetativeMethod::Runner, 1.0, 0.0),
            0.0
        );
    }

    #[test]
    fn resource_limited_half_resources() {
        let actual = resource_limited_ramets(VegetativeMethod::Rhizome, 0.5, 0.8);
        let expected = base_ramet_rate(VegetativeMethod::Rhizome) * 0.5;
        assert!((actual - expected).abs() < 0.01);
    }

    #[test]
    fn stress_takes_minimum() {
        // N is more limiting
        let actual = resource_limited_ramets(VegetativeMethod::Runner, 0.9, 0.3);
        let expected = base_ramet_rate(VegetativeMethod::Runner) * 0.3;
        assert!((actual - expected).abs() < 0.01);
    }

    // --- clonal area ---

    #[test]
    fn clonal_area_zero_years() {
        assert_eq!(clonal_area_m2(VegetativeMethod::Rhizome, 0.0), 0.0);
    }

    #[test]
    fn clonal_area_one_year() {
        let area = clonal_area_m2(VegetativeMethod::Rhizome, 1.0);
        // π × 3² ≈ 28.27
        assert!((area - 28.27).abs() < 0.1, "got {area}");
    }

    #[test]
    fn clonal_area_grows_quadratically() {
        let a1 = clonal_area_m2(VegetativeMethod::Runner, 1.0);
        let a2 = clonal_area_m2(VegetativeMethod::Runner, 2.0);
        // area ∝ years², so a2/a1 ≈ 4
        assert!((a2 / a1 - 4.0).abs() < 0.1, "ratio = {}", a2 / a1);
    }

    #[test]
    fn root_sprout_largest_area() {
        let rs = clonal_area_m2(VegetativeMethod::RootSprout, 5.0);
        let rn = clonal_area_m2(VegetativeMethod::Runner, 5.0);
        assert!(rs > rn);
    }

    // --- parent cost ---

    #[test]
    fn parent_cost_basic() {
        let cost = parent_cost_kg(100.0, VegetativeMethod::Runner, 5.0);
        // 100 × 0.02 × 5 = 10 kg
        assert!((cost - 10.0).abs() < 0.1, "got {cost}");
    }

    #[test]
    fn parent_cost_zero_ramets() {
        assert_eq!(parent_cost_kg(100.0, VegetativeMethod::Runner, 0.0), 0.0);
    }

    #[test]
    fn parent_cost_zero_biomass() {
        assert_eq!(parent_cost_kg(0.0, VegetativeMethod::Runner, 5.0), 0.0);
    }

    #[test]
    fn parent_cost_capped_at_parent_mass() {
        // 10 kg parent, 100 ramets at 0.05 each = 50 kg → capped at 10
        let cost = parent_cost_kg(10.0, VegetativeMethod::RootSprout, 100.0);
        assert!(
            (cost - 10.0).abs() < 0.01,
            "should cap at parent mass, got {cost}"
        );
    }

    #[test]
    fn root_sprout_costs_more_per_ramet() {
        let rs = parent_cost_kg(100.0, VegetativeMethod::RootSprout, 1.0);
        let rn = parent_cost_kg(100.0, VegetativeMethod::Runner, 1.0);
        assert!(rs > rn);
    }
}
