//! Mycorrhizal networks — plant-fungal symbiosis for nutrient exchange.
//!
//! ~90% of terrestrial plants form mycorrhizal associations. The fungal
//! partner extends the plant's effective root system, enhancing nutrient
//! (especially phosphorus and nitrogen) uptake in exchange for carbon.

use serde::{Deserialize, Serialize};

/// Mycorrhizal association type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MycorrhizalType {
    /// Ectomycorrhizal — fungal sheath around root tips, Hartig net.
    /// Dominant in temperate/boreal forests (oak, pine, birch, beech).
    /// Strong N and P enhancement, extensive hyphal networks.
    Ectomycorrhizal,
    /// Arbuscular mycorrhizal (AM) — fungal arbuscules inside root cells.
    /// Most common type (~80% of plant species). Grasses, herbs, tropical trees.
    /// Primary benefit is phosphorus uptake.
    Arbuscular,
    /// Ericoid mycorrhizal — specialized for heathland/acidic soils.
    /// Ericaceae (heather, blueberry, rhododendron).
    /// Accesses organic N/P that other types cannot.
    Ericoid,
}

/// Nutrient uptake enhancement factor from mycorrhizal association (dimensionless multiplier).
///
/// How much the fungal partner increases effective nutrient uptake beyond
/// what roots alone can achieve.
///
/// Returns `(nitrogen_multiplier, phosphorus_multiplier)`.
///
/// Based on meta-analysis by Hoeksema et al. (2010) — mycorrhizal plants
/// show 20–80% increases in nutrient uptake depending on type and nutrient.
///
/// - `myc_type` — mycorrhizal association type
#[must_use]
pub fn nutrient_enhancement(myc_type: MycorrhizalType) -> (f32, f32) {
    let (n_mult, p_mult) = match myc_type {
        // ECM: strong N (organic N access) and P enhancement
        MycorrhizalType::Ectomycorrhizal => (1.6, 1.5),
        // AM: moderate N, strong P (hyphal P transport)
        MycorrhizalType::Arbuscular => (1.3, 1.8),
        // Ericoid: strong organic N access in acidic soils
        MycorrhizalType::Ericoid => (1.8, 1.4),
    };
    tracing::trace!(?myc_type, n_mult, p_mult, "nutrient_enhancement");
    (n_mult, p_mult)
}

/// Carbon cost of maintaining mycorrhizal association (fraction of NPP).
///
/// Plants allocate 10–30% of photosynthate to fungal partners.
/// This is the "price" of enhanced nutrient uptake.
///
/// - `myc_type` — mycorrhizal association type
#[must_use]
#[inline]
pub fn carbon_cost_fraction(myc_type: MycorrhizalType) -> f32 {
    let cost = match myc_type {
        // ECM: high cost — extensive external mycelium
        MycorrhizalType::Ectomycorrhizal => 0.20,
        // AM: moderate cost
        MycorrhizalType::Arbuscular => 0.15,
        // Ericoid: lower cost — smaller hyphal networks
        MycorrhizalType::Ericoid => 0.12,
    };
    tracing::trace!(?myc_type, cost, "carbon_cost_fraction");
    cost
}

/// Colonization rate — fraction of root length colonized (0.0–1.0).
///
/// Mycorrhizal colonization depends on soil phosphorus availability.
/// Low P → high colonization (plant invests more in fungi).
/// High P → low colonization (plant can get P alone).
///
/// `colonization = max_col × (1.0 - soil_p_saturation)`
///
/// Max colonization: ECM 0.7, AM 0.8, Ericoid 0.6.
///
/// - `myc_type` — mycorrhizal association type
/// - `soil_p_saturation` — soil phosphorus availability relative to demand (0.0–1.0)
#[must_use]
pub fn colonization_rate(myc_type: MycorrhizalType, soil_p_saturation: f32) -> f32 {
    let max_col = match myc_type {
        MycorrhizalType::Ectomycorrhizal => 0.7,
        MycorrhizalType::Arbuscular => 0.8,
        MycorrhizalType::Ericoid => 0.6,
    };
    let p_sat = soil_p_saturation.clamp(0.0, 1.0);
    let col = max_col * (1.0 - p_sat);
    tracing::trace!(
        ?myc_type,
        soil_p_saturation,
        max_col,
        col,
        "colonization_rate"
    );
    col
}

/// Effective nitrogen uptake with mycorrhizal enhancement (kg N/m²/day).
///
/// Scales base uptake by the nitrogen enhancement factor, weighted by
/// colonization rate. Uncolonized roots get no fungal benefit.
///
/// `effective = base_uptake × (1.0 + (n_mult - 1.0) × colonization)`
///
/// At 0% colonization: effective = base_uptake (no benefit).
/// At 100% colonization: effective = base_uptake × n_mult (full benefit).
///
/// - `base_uptake` — nitrogen uptake without fungi (kg N/m²/day)
/// - `myc_type` — mycorrhizal association type
/// - `colonization` — fraction of roots colonized (0.0–1.0)
#[must_use]
#[inline]
pub fn enhanced_n_uptake(base_uptake: f32, myc_type: MycorrhizalType, colonization: f32) -> f32 {
    if base_uptake <= 0.0 {
        return 0.0;
    }
    let (n_mult, _) = nutrient_enhancement(myc_type);
    let col = colonization.clamp(0.0, 1.0);
    let effective = base_uptake * (1.0 + (n_mult - 1.0) * col);
    tracing::trace!(
        base_uptake,
        ?myc_type,
        colonization,
        n_mult,
        effective,
        "enhanced_n_uptake"
    );
    effective
}

/// Net benefit of mycorrhizal association (dimensionless, >1.0 = beneficial).
///
/// Compares nutrient gain against carbon cost. When soil nutrients are
/// abundant, the cost may exceed the benefit.
///
/// `benefit = (n_enhancement × colonization) / carbon_cost`
///
/// Ratio > 1.0: association is beneficial (more gain than cost).
/// Ratio < 1.0: association is costly (plant would grow faster without fungi).
///
/// - `myc_type` — mycorrhizal association type
/// - `colonization` — fraction of roots colonized (0.0–1.0)
/// - `nutrient_limitation` — how N-limited the plant is (0.0=no limit, 1.0=severe)
#[must_use]
pub fn net_benefit_ratio(
    myc_type: MycorrhizalType,
    colonization: f32,
    nutrient_limitation: f32,
) -> f32 {
    let col = colonization.clamp(0.0, 1.0);
    let n_lim = nutrient_limitation.clamp(0.0, 1.0);
    let cost = carbon_cost_fraction(myc_type);
    if cost <= 0.0 {
        return 1.0;
    }
    let (n_mult, _) = nutrient_enhancement(myc_type);
    // Benefit scales with both enhancement and how much the plant needs it
    let benefit = (n_mult - 1.0) * col * n_lim;
    let ratio = benefit / cost;
    tracing::trace!(
        ?myc_type,
        colonization,
        nutrient_limitation,
        benefit,
        cost,
        ratio,
        "net_benefit_ratio"
    );
    ratio
}

/// Hyphal network exploration distance (meters from root surface).
///
/// How far the fungal hyphae extend into soil beyond the root zone,
/// accessing nutrients the roots alone cannot reach.
///
/// - `myc_type` — mycorrhizal association type
#[must_use]
#[inline]
pub fn hyphal_reach_m(myc_type: MycorrhizalType) -> f32 {
    let reach = match myc_type {
        MycorrhizalType::Ectomycorrhizal => 3.0, // extensive external mycelium
        MycorrhizalType::Arbuscular => 1.5,      // moderate hyphal extent
        MycorrhizalType::Ericoid => 0.5,         // limited hyphal extent
    };
    tracing::trace!(?myc_type, reach, "hyphal_reach_m");
    reach
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- nutrient enhancement ---

    #[test]
    fn all_enhancements_above_one() {
        for myc in [
            MycorrhizalType::Ectomycorrhizal,
            MycorrhizalType::Arbuscular,
            MycorrhizalType::Ericoid,
        ] {
            let (n, p) = nutrient_enhancement(myc);
            assert!(n > 1.0, "{myc:?} N enhancement should be >1.0");
            assert!(p > 1.0, "{myc:?} P enhancement should be >1.0");
        }
    }

    #[test]
    fn am_strongest_phosphorus() {
        let (_, am_p) = nutrient_enhancement(MycorrhizalType::Arbuscular);
        let (_, ecm_p) = nutrient_enhancement(MycorrhizalType::Ectomycorrhizal);
        assert!(am_p > ecm_p, "AM should be strongest for P");
    }

    #[test]
    fn ericoid_strongest_nitrogen() {
        let (er_n, _) = nutrient_enhancement(MycorrhizalType::Ericoid);
        let (am_n, _) = nutrient_enhancement(MycorrhizalType::Arbuscular);
        assert!(er_n > am_n, "ericoid should be strongest for N");
    }

    // --- carbon cost ---

    #[test]
    fn costs_in_valid_range() {
        for myc in [
            MycorrhizalType::Ectomycorrhizal,
            MycorrhizalType::Arbuscular,
            MycorrhizalType::Ericoid,
        ] {
            let c = carbon_cost_fraction(myc);
            assert!((0.05..=0.35).contains(&c), "{myc:?}: {c}");
        }
    }

    #[test]
    fn ecm_most_expensive() {
        assert!(
            carbon_cost_fraction(MycorrhizalType::Ectomycorrhizal)
                > carbon_cost_fraction(MycorrhizalType::Ericoid)
        );
    }

    // --- colonization ---

    #[test]
    fn colonization_high_when_p_low() {
        let col = colonization_rate(MycorrhizalType::Arbuscular, 0.0);
        assert!((col - 0.8).abs() < 0.01, "max colonization at zero P");
    }

    #[test]
    fn colonization_zero_when_p_saturated() {
        let col = colonization_rate(MycorrhizalType::Arbuscular, 1.0);
        assert_eq!(col, 0.0, "no colonization when P is abundant");
    }

    #[test]
    fn colonization_decreases_with_p() {
        let low_p = colonization_rate(MycorrhizalType::Ectomycorrhizal, 0.2);
        let high_p = colonization_rate(MycorrhizalType::Ectomycorrhizal, 0.8);
        assert!(low_p > high_p);
    }

    // --- enhanced uptake ---

    #[test]
    fn enhanced_uptake_at_zero_colonization() {
        let base = 0.001;
        let enhanced = enhanced_n_uptake(base, MycorrhizalType::Arbuscular, 0.0);
        assert!(
            (enhanced - base).abs() < 0.0001,
            "no colonization → base uptake"
        );
    }

    #[test]
    fn enhanced_uptake_at_full_colonization() {
        let base = 0.001;
        let enhanced = enhanced_n_uptake(base, MycorrhizalType::Ectomycorrhizal, 1.0);
        let (n_mult, _) = nutrient_enhancement(MycorrhizalType::Ectomycorrhizal);
        let expected = base * n_mult;
        assert!((enhanced - expected).abs() < 0.0001, "got {enhanced}");
    }

    #[test]
    fn enhanced_uptake_zero_base() {
        assert_eq!(
            enhanced_n_uptake(0.0, MycorrhizalType::Arbuscular, 1.0),
            0.0
        );
    }

    #[test]
    fn enhanced_always_gte_base() {
        let base = 0.001;
        for myc in [
            MycorrhizalType::Ectomycorrhizal,
            MycorrhizalType::Arbuscular,
            MycorrhizalType::Ericoid,
        ] {
            let enhanced = enhanced_n_uptake(base, myc, 0.5);
            assert!(enhanced >= base, "{myc:?}");
        }
    }

    // --- net benefit ---

    #[test]
    fn net_benefit_high_when_nutrient_limited() {
        let ratio = net_benefit_ratio(MycorrhizalType::Ectomycorrhizal, 0.7, 0.9);
        assert!(
            ratio > 1.0,
            "should be beneficial when N-limited, got {ratio}"
        );
    }

    #[test]
    fn net_benefit_low_when_nutrients_abundant() {
        let ratio = net_benefit_ratio(MycorrhizalType::Ectomycorrhizal, 0.7, 0.1);
        assert!(ratio < 1.0, "should be costly when N-abundant, got {ratio}");
    }

    #[test]
    fn net_benefit_zero_colonization() {
        let ratio = net_benefit_ratio(MycorrhizalType::Arbuscular, 0.0, 1.0);
        assert_eq!(ratio, 0.0, "no colonization → no benefit");
    }

    // --- hyphal reach ---

    #[test]
    fn ecm_longest_reach() {
        assert!(
            hyphal_reach_m(MycorrhizalType::Ectomycorrhizal)
                > hyphal_reach_m(MycorrhizalType::Ericoid)
        );
    }

    #[test]
    fn all_reaches_positive() {
        for myc in [
            MycorrhizalType::Ectomycorrhizal,
            MycorrhizalType::Arbuscular,
            MycorrhizalType::Ericoid,
        ] {
            assert!(hyphal_reach_m(myc) > 0.0, "{myc:?}");
        }
    }
}
