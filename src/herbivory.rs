//! Herbivory pressure — grazing, browsing, and plant compensatory responses.
//!
//! Models biomass removal by herbivores and the plant's ability to regrow.
//! Connects to jantu (creature AI) for herbivore-plant interactions.

use serde::{Deserialize, Serialize};

/// Herbivory type — determines which plant organs are consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HerbivoryType {
    /// Grazing — removal of leaf/shoot tissue (grasses, herbs).
    /// Primarily affects leaf biomass. Examples: cattle, sheep, grasshoppers.
    Grazing,
    /// Browsing — removal of woody shoots, bark, buds.
    /// Affects leaf + stem biomass. Examples: deer, goats, elephants.
    Browsing,
    /// Frugivory — consumption of fruits/seeds.
    /// Affects reproductive biomass. Examples: birds, bats, primates.
    Frugivory,
    /// Root herbivory — below-ground consumption.
    /// Affects root biomass. Examples: nematodes, beetle larvae.
    RootFeeding,
}

/// Biomass removal fractions per herbivory type.
///
/// Returns `(leaf_fraction, stem_fraction, root_fraction, reproductive_fraction)`
/// representing what proportion of each pool is accessible to this herbivore type.
///
/// The actual removal depends on intensity (see `biomass_removal`).
///
/// - `herbivory_type` — type of herbivory
#[must_use]
pub fn organ_vulnerability(herbivory_type: HerbivoryType) -> (f32, f32, f32, f32) {
    let fracs = match herbivory_type {
        // Grazers eat leaves/shoots
        HerbivoryType::Grazing => (0.8, 0.1, 0.0, 0.1),
        // Browsers eat leaves + woody parts
        HerbivoryType::Browsing => (0.5, 0.3, 0.0, 0.2),
        // Frugivores target reproductive organs
        HerbivoryType::Frugivory => (0.0, 0.0, 0.0, 1.0),
        // Root feeders target below-ground
        HerbivoryType::RootFeeding => (0.0, 0.0, 1.0, 0.0),
    };
    tracing::trace!(?herbivory_type, ?fracs, "organ_vulnerability");
    fracs
}

/// Biomass removed by herbivory (kg per organ).
///
/// Returns `(leaf_removed, stem_removed, root_removed, repro_removed)` in kg.
///
/// Intensity (0.0–1.0) represents the fraction of *vulnerable* biomass consumed.
/// Light grazing: 0.1–0.3, moderate: 0.3–0.5, heavy: 0.5–0.8.
///
/// - `leaf_kg` — current leaf biomass (kg)
/// - `stem_kg` — current stem biomass (kg)
/// - `root_kg` — current root biomass (kg)
/// - `reproductive_kg` — current reproductive biomass (kg)
/// - `herbivory_type` — type of herbivory
/// - `intensity` — consumption intensity (0.0–1.0)
#[must_use]
pub fn biomass_removal(
    leaf_kg: f32,
    stem_kg: f32,
    root_kg: f32,
    reproductive_kg: f32,
    herbivory_type: HerbivoryType,
    intensity: f32,
) -> (f32, f32, f32, f32) {
    let intensity = intensity.clamp(0.0, 1.0);
    if intensity <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let (vl, vs, vr, vrp) = organ_vulnerability(herbivory_type);
    let removed = (
        leaf_kg * vl * intensity,
        stem_kg * vs * intensity,
        root_kg * vr * intensity,
        reproductive_kg * vrp * intensity,
    );
    tracing::trace!(
        ?herbivory_type,
        intensity,
        leaf_removed = removed.0,
        stem_removed = removed.1,
        root_removed = removed.2,
        repro_removed = removed.3,
        "biomass_removal"
    );
    removed
}

/// Total biomass removed across all organs (kg).
///
/// Convenience wrapper around `biomass_removal`.
///
/// - `leaf_kg` — current leaf biomass (kg)
/// - `stem_kg` — current stem biomass (kg)
/// - `root_kg` — current root biomass (kg)
/// - `reproductive_kg` — current reproductive biomass (kg)
/// - `herbivory_type` — type of herbivory
/// - `intensity` — consumption intensity (0.0–1.0)
#[must_use]
pub fn total_biomass_removed(
    leaf_kg: f32,
    stem_kg: f32,
    root_kg: f32,
    reproductive_kg: f32,
    herbivory_type: HerbivoryType,
    intensity: f32,
) -> f32 {
    let (l, s, r, rp) = biomass_removal(
        leaf_kg,
        stem_kg,
        root_kg,
        reproductive_kg,
        herbivory_type,
        intensity,
    );
    l + s + r + rp
}

/// Compensatory growth factor (0.0–2.0+).
///
/// Plants can partially or fully compensate for herbivory by reallocating
/// resources to regrowth. Light defoliation often stimulates growth
/// (overcompensation), while severe defoliation suppresses it.
///
/// `factor = 1.0 + compensation × (1.0 - 2.0 × defoliation_fraction)`
///
/// At 0% defoliation: factor = 1.0 + compensation (slight stimulation)
/// At 25% defoliation: factor = 1.0 + 0.5 × compensation (moderate stimulation)
/// At 50% defoliation: factor = 1.0 (neutral)
/// At 100% defoliation: factor = 1.0 - compensation (suppressed)
///
/// Compensation capacity varies: grasses 0.3–0.5, trees 0.1–0.2.
///
/// - `defoliation_fraction` — fraction of leaf biomass removed (0.0–1.0)
/// - `compensation_capacity` — species ability to compensate (0.0–0.5)
#[must_use]
#[inline]
pub fn compensatory_growth_factor(defoliation_fraction: f32, compensation_capacity: f32) -> f32 {
    let defol = defoliation_fraction.clamp(0.0, 1.0);
    let comp = compensation_capacity.clamp(0.0, 0.5);
    let factor = 1.0 + comp * (1.0 - 2.0 * defol);
    tracing::trace!(
        defoliation_fraction,
        compensation_capacity,
        factor,
        "compensatory_growth_factor"
    );
    factor.max(0.0)
}

/// Herbivory-induced mortality probability (0.0–1.0).
///
/// Severe defoliation can kill plants, especially seedlings and those
/// with low reserves. Mortality increases sharply above ~70% defoliation.
///
/// `mortality = ((defoliation - 0.7) / 0.3)² × vulnerability`
///
/// Below 70% defoliation: mortality = 0.
/// Vulnerability: seedling 1.0, mature tree 0.3, grass 0.1 (resprouts easily).
///
/// - `defoliation_fraction` — fraction of total biomass removed (0.0–1.0)
/// - `vulnerability` — species/stage vulnerability (0.0–1.0)
#[must_use]
#[inline]
pub fn herbivory_mortality(defoliation_fraction: f32, vulnerability: f32) -> f32 {
    let defol = defoliation_fraction.clamp(0.0, 1.0);
    if defol <= 0.7 {
        return 0.0;
    }
    let vuln = vulnerability.clamp(0.0, 1.0);
    let excess = (defol - 0.7) / 0.3;
    let mort = (excess * excess * vuln).clamp(0.0, 1.0);
    tracing::trace!(
        defoliation_fraction,
        vulnerability,
        mort,
        "herbivory_mortality"
    );
    mort
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- organ_vulnerability ---

    #[test]
    fn grazing_targets_leaves() {
        let (l, s, r, rp) = organ_vulnerability(HerbivoryType::Grazing);
        assert!(l > s);
        assert!(l > r);
        assert!(l > rp);
    }

    #[test]
    fn browsing_targets_leaves_and_stems() {
        let (l, s, _, _) = organ_vulnerability(HerbivoryType::Browsing);
        assert!(l > 0.0);
        assert!(s > 0.0);
    }

    #[test]
    fn frugivory_targets_reproductive() {
        let (l, s, r, rp) = organ_vulnerability(HerbivoryType::Frugivory);
        assert_eq!(l, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(r, 0.0);
        assert_eq!(rp, 1.0);
    }

    #[test]
    fn root_feeding_targets_roots() {
        let (l, s, r, rp) = organ_vulnerability(HerbivoryType::RootFeeding);
        assert_eq!(l, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(r, 1.0);
        assert_eq!(rp, 0.0);
    }

    #[test]
    fn vulnerability_fractions_sum_to_one() {
        for ht in [
            HerbivoryType::Grazing,
            HerbivoryType::Browsing,
            HerbivoryType::Frugivory,
            HerbivoryType::RootFeeding,
        ] {
            let (l, s, r, rp) = organ_vulnerability(ht);
            assert!(
                (l + s + r + rp - 1.0).abs() < 0.01,
                "{ht:?} fractions don't sum to 1.0"
            );
        }
    }

    // --- biomass_removal ---

    #[test]
    fn removal_zero_intensity() {
        let (l, s, r, rp) = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Grazing, 0.0);
        assert_eq!(l + s + r + rp, 0.0);
    }

    #[test]
    fn removal_grazing_mostly_leaves() {
        let (l, s, r, _rp) = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Grazing, 0.5);
        assert!(l > s, "grazing should remove more leaf than stem");
        assert_eq!(r, 0.0, "grazing doesn't touch roots");
    }

    #[test]
    fn removal_full_intensity() {
        let (_, _, _, rp) = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Frugivory, 1.0);
        assert!(
            (rp - 10.0).abs() < 0.01,
            "full frugivory should take all repro"
        );
    }

    #[test]
    fn removal_clamped_above_one() {
        let a = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Grazing, 1.0);
        let b = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Grazing, 2.0);
        assert_eq!(a, b, "intensity >1.0 should clamp to 1.0");
    }

    #[test]
    fn total_removal_sums_organs() {
        let total = total_biomass_removed(50.0, 100.0, 30.0, 10.0, HerbivoryType::Browsing, 0.5);
        let (l, s, r, rp) = biomass_removal(50.0, 100.0, 30.0, 10.0, HerbivoryType::Browsing, 0.5);
        assert!((total - (l + s + r + rp)).abs() < 0.01);
    }

    // --- compensatory growth ---

    #[test]
    fn compensatory_no_defoliation() {
        let f = compensatory_growth_factor(0.0, 0.3);
        assert!(f > 1.0, "no defoliation → slight stimulation, got {f}");
    }

    #[test]
    fn compensatory_light_grazing() {
        let f = compensatory_growth_factor(0.25, 0.3);
        assert!(f > 1.0, "light grazing → overcompensation, got {f}");
    }

    #[test]
    fn compensatory_half_defoliation() {
        let f = compensatory_growth_factor(0.5, 0.3);
        assert!((f - 1.0).abs() < 0.01, "50% defoliation → neutral, got {f}");
    }

    #[test]
    fn compensatory_heavy_defoliation() {
        let f = compensatory_growth_factor(0.9, 0.3);
        assert!(f < 1.0, "heavy defoliation → suppressed, got {f}");
    }

    #[test]
    fn compensatory_zero_capacity() {
        let f = compensatory_growth_factor(0.25, 0.0);
        assert!((f - 1.0).abs() < 0.01, "zero capacity → always 1.0");
    }

    #[test]
    fn grass_compensates_better_than_tree() {
        let grass = compensatory_growth_factor(0.3, 0.4);
        let tree = compensatory_growth_factor(0.3, 0.15);
        assert!(grass > tree, "grass should compensate better");
    }

    // --- herbivory mortality ---

    #[test]
    fn mortality_light_defoliation_zero() {
        assert_eq!(herbivory_mortality(0.5, 1.0), 0.0);
    }

    #[test]
    fn mortality_at_threshold() {
        assert_eq!(herbivory_mortality(0.7, 1.0), 0.0);
    }

    #[test]
    fn mortality_severe_defoliation() {
        let m = herbivory_mortality(0.9, 1.0);
        assert!(
            m > 0.3,
            "90% defoliation should cause high mortality, got {m}"
        );
    }

    #[test]
    fn mortality_total_defoliation() {
        let m = herbivory_mortality(1.0, 1.0);
        assert!(
            (m - 1.0).abs() < 0.01,
            "100% defoliation + full vulnerability = death"
        );
    }

    #[test]
    fn mortality_low_vulnerability() {
        let high_vuln = herbivory_mortality(0.9, 1.0);
        let low_vuln = herbivory_mortality(0.9, 0.1);
        assert!(
            low_vuln < high_vuln,
            "grass should survive better than seedling"
        );
    }

    #[test]
    fn mortality_zero_vulnerability() {
        assert_eq!(herbivory_mortality(1.0, 0.0), 0.0);
    }
}
