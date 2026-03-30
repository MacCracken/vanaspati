//! Plant Functional Types (PFTs) — standardized species parameterization.
//!
//! PFTs group species with similar ecological strategies into parameter sets.
//! This is the standard approach in all major vegetation models (LPJ-GUESS,
//! JULES, ED2). Consumers use PFTs to configure growth, photosynthesis,
//! allocation, and mortality without specifying dozens of individual parameters.

use crate::lai::LeafHabit;
use crate::photosynthesis::PhotosynthesisPathway;
use crate::succession::SuccessionalStage;
use serde::{Deserialize, Serialize};

/// Plant functional type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PftType {
    /// Tropical broadleaf evergreen — rainforest canopy trees.
    TropicalBroadleafEvergreen,
    /// Temperate broadleaf deciduous — oak, beech, maple.
    TemperateBroadleafDeciduous,
    /// Temperate needleleaf evergreen — pine, spruce, fir.
    TemperateNeedleleafEvergreen,
    /// Boreal needleleaf evergreen — black spruce, jack pine.
    BorealNeedleleafEvergreen,
    /// C3 grass — temperate grasses, wheat, rice.
    C3Grass,
    /// C4 grass — tropical/warm-season grasses, corn, sugarcane.
    C4Grass,
    /// Shrub — Mediterranean, chaparral, heath.
    Shrub,
}

/// Complete parameter set for a plant functional type.
///
/// All the numbers a consumer needs to configure vanaspati's modules
/// for a given species group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PftParams {
    /// PFT identifier.
    pub pft_type: PftType,

    // ── Photosynthesis ──────────────────────────────────────────────
    /// Photosynthesis pathway (C3/C4/CAM).
    pub pathway: PhotosynthesisPathway,
    /// Maximum photosynthesis rate at light saturation (µmol CO₂/m²/s).
    pub pmax: f32,
    /// Quantum yield (mol CO₂ / mol photons).
    pub quantum_yield: f32,
    /// Optimal temperature for photosynthesis (°C).
    pub temp_optimum: f32,

    // ── Growth ───────────────────────────────────────────────────────
    /// Maximum height (meters).
    pub max_height: f32,
    /// Relative growth rate (per day).
    pub growth_rate: f32,
    /// Successional strategy.
    pub successional_stage: SuccessionalStage,

    // ── Leaf ─────────────────────────────────────────────────────────
    /// Leaf habit (deciduous/evergreen).
    pub leaf_habit: LeafHabit,
    /// Specific leaf area (m² leaf / kg leaf).
    pub sla: f32,
    /// Maximum leaf area index (m² leaf / m² ground).
    pub lai_max: f32,
    /// Leaf N concentration (kg N / kg leaf).
    pub leaf_n: f32,

    // ── Allocation ──────────────────────────────────────────────────
    /// Fraction to leaves (0.0–1.0).
    pub alloc_leaf: f32,
    /// Fraction to stem (0.0–1.0).
    pub alloc_stem: f32,
    /// Fraction to roots (0.0–1.0).
    pub alloc_root: f32,

    // ── Root ─────────────────────────────────────────────────────────
    /// Maximum root depth (meters).
    pub root_depth: f32,

    // ── Mortality & turnover ─────────────────────────────────────────
    /// Maximum lifespan (years).
    pub lifespan_years: f32,
    /// Annual leaf turnover rate (fraction, 1.0 = all leaves replaced yearly).
    pub leaf_turnover: f32,
    /// Annual fine root turnover rate (fraction).
    pub root_turnover: f32,
    /// Wood N concentration (kg N / kg wood).
    pub wood_n: f32,
    /// Cold hardiness threshold (°C).
    pub frost_threshold: f32,

    // ── Critical N ───────────────────────────────────────────────────
    /// Critical N concentration for unstressed growth (kg N / kg biomass).
    pub critical_n: f32,
}

impl PftParams {
    /// Get parameters for a PFT type.
    #[must_use]
    pub fn from_type(pft_type: PftType) -> Self {
        match pft_type {
            PftType::TropicalBroadleafEvergreen => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 18.0,
                quantum_yield: 0.05,
                temp_optimum: 28.0,
                max_height: 40.0,
                growth_rate: 0.008,
                successional_stage: SuccessionalStage::Climax,
                leaf_habit: LeafHabit::Evergreen,
                sla: 18.0,
                lai_max: 8.0,
                leaf_n: 0.020,
                alloc_leaf: 0.25,
                alloc_stem: 0.45,
                alloc_root: 0.25,
                root_depth: 3.0,
                lifespan_years: 500.0,
                leaf_turnover: 0.5,
                root_turnover: 0.5,
                wood_n: 0.003,
                frost_threshold: 2.0, // very frost-sensitive
                critical_n: 0.012,
            },
            PftType::TemperateBroadleafDeciduous => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 20.0,
                quantum_yield: 0.05,
                temp_optimum: 25.0,
                max_height: 25.0,
                growth_rate: 0.005,
                successional_stage: SuccessionalStage::MidSuccessional,
                leaf_habit: LeafHabit::Deciduous,
                sla: 25.0,
                lai_max: 7.0,
                leaf_n: 0.025,
                alloc_leaf: 0.30,
                alloc_stem: 0.35,
                alloc_root: 0.30,
                root_depth: 2.5,
                lifespan_years: 200.0,
                leaf_turnover: 1.0, // all leaves replaced annually
                root_turnover: 0.6,
                wood_n: 0.004,
                frost_threshold: -15.0,
                critical_n: 0.012,
            },
            PftType::TemperateNeedleleafEvergreen => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 12.0,
                quantum_yield: 0.04,
                temp_optimum: 22.0,
                max_height: 30.0,
                growth_rate: 0.004,
                successional_stage: SuccessionalStage::Climax,
                leaf_habit: LeafHabit::Evergreen,
                sla: 8.0,
                lai_max: 10.0,
                leaf_n: 0.012,
                alloc_leaf: 0.20,
                alloc_stem: 0.45,
                alloc_root: 0.30,
                root_depth: 2.0,
                lifespan_years: 400.0,
                leaf_turnover: 0.25, // 4-year needle retention
                root_turnover: 0.5,
                wood_n: 0.003,
                frost_threshold: -30.0,
                critical_n: 0.008,
            },
            PftType::BorealNeedleleafEvergreen => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 8.0,
                quantum_yield: 0.04,
                temp_optimum: 18.0,
                max_height: 20.0,
                growth_rate: 0.003,
                successional_stage: SuccessionalStage::Climax,
                leaf_habit: LeafHabit::Evergreen,
                sla: 6.0,
                lai_max: 8.0,
                leaf_n: 0.010,
                alloc_leaf: 0.20,
                alloc_stem: 0.40,
                alloc_root: 0.35,
                root_depth: 1.5,
                lifespan_years: 300.0,
                leaf_turnover: 0.15, // 6-7 year needle retention
                root_turnover: 0.4,
                wood_n: 0.002,
                frost_threshold: -45.0,
                critical_n: 0.008,
            },
            PftType::C3Grass => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 20.0,
                quantum_yield: 0.05,
                temp_optimum: 22.0,
                max_height: 0.8,
                growth_rate: 0.08,
                successional_stage: SuccessionalStage::Pioneer,
                leaf_habit: LeafHabit::Deciduous,
                sla: 30.0,
                lai_max: 4.0,
                leaf_n: 0.025,
                alloc_leaf: 0.45,
                alloc_stem: 0.10,
                alloc_root: 0.40,
                root_depth: 0.5,
                lifespan_years: 5.0,
                leaf_turnover: 1.0,
                root_turnover: 0.8,
                wood_n: 0.005, // no real wood — stem N
                frost_threshold: -10.0,
                critical_n: 0.015,
            },
            PftType::C4Grass => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C4,
                pmax: 40.0,
                quantum_yield: 0.06,
                temp_optimum: 32.0,
                max_height: 1.5,
                growth_rate: 0.10,
                successional_stage: SuccessionalStage::Pioneer,
                leaf_habit: LeafHabit::Deciduous,
                sla: 28.0,
                lai_max: 5.0,
                leaf_n: 0.015,
                alloc_leaf: 0.40,
                alloc_stem: 0.15,
                alloc_root: 0.40,
                root_depth: 1.0,
                lifespan_years: 3.0,
                leaf_turnover: 1.0,
                root_turnover: 0.8,
                wood_n: 0.004,
                frost_threshold: -2.0, // very frost-sensitive
                critical_n: 0.010,
            },
            PftType::Shrub => Self {
                pft_type,
                pathway: PhotosynthesisPathway::C3,
                pmax: 15.0,
                quantum_yield: 0.05,
                temp_optimum: 25.0,
                max_height: 3.0,
                growth_rate: 0.02,
                successional_stage: SuccessionalStage::Pioneer,
                leaf_habit: LeafHabit::Evergreen,
                sla: 12.0,
                lai_max: 4.0,
                leaf_n: 0.018,
                alloc_leaf: 0.30,
                alloc_stem: 0.30,
                alloc_root: 0.35,
                root_depth: 2.0,
                lifespan_years: 50.0,
                leaf_turnover: 0.4,
                root_turnover: 0.5,
                wood_n: 0.004,
                frost_threshold: -10.0,
                critical_n: 0.010,
            },
        }
    }

    /// Allocation fractions sum (should be ≤1.0, remainder is reproductive).
    #[must_use]
    #[inline]
    pub fn alloc_reproductive(&self) -> f32 {
        (1.0 - self.alloc_leaf - self.alloc_stem - self.alloc_root).max(0.0)
    }

    /// Whether this PFT is woody (trees/shrubs vs grasses).
    #[must_use]
    #[inline]
    pub fn is_woody(&self) -> bool {
        self.max_height > 2.0
    }

    /// Whether this PFT is a conifer.
    #[must_use]
    #[inline]
    pub fn is_conifer(&self) -> bool {
        matches!(
            self.pft_type,
            PftType::TemperateNeedleleafEvergreen | PftType::BorealNeedleleafEvergreen
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pfts_construct() {
        for pft in [
            PftType::TropicalBroadleafEvergreen,
            PftType::TemperateBroadleafDeciduous,
            PftType::TemperateNeedleleafEvergreen,
            PftType::BorealNeedleleafEvergreen,
            PftType::C3Grass,
            PftType::C4Grass,
            PftType::Shrub,
        ] {
            let p = PftParams::from_type(pft);
            assert!(p.pmax > 0.0, "{pft:?} pmax");
            assert!(p.max_height > 0.0, "{pft:?} max_height");
            assert!(p.sla > 0.0, "{pft:?} sla");
        }
    }

    #[test]
    fn allocation_sums_to_one() {
        for pft in [
            PftType::TropicalBroadleafEvergreen,
            PftType::TemperateBroadleafDeciduous,
            PftType::TemperateNeedleleafEvergreen,
            PftType::BorealNeedleleafEvergreen,
            PftType::C3Grass,
            PftType::C4Grass,
            PftType::Shrub,
        ] {
            let p = PftParams::from_type(pft);
            let sum = p.alloc_leaf + p.alloc_stem + p.alloc_root + p.alloc_reproductive();
            assert!(
                (sum - 1.0).abs() < 0.01,
                "{pft:?}: allocations sum to {sum}"
            );
        }
    }

    #[test]
    fn c4_grass_has_c4_pathway() {
        let p = PftParams::from_type(PftType::C4Grass);
        assert_eq!(p.pathway, PhotosynthesisPathway::C4);
    }

    #[test]
    fn c4_higher_pmax_than_c3() {
        let c3 = PftParams::from_type(PftType::C3Grass);
        let c4 = PftParams::from_type(PftType::C4Grass);
        assert!(c4.pmax > c3.pmax);
    }

    #[test]
    fn tropical_tallest() {
        let trop = PftParams::from_type(PftType::TropicalBroadleafEvergreen);
        let temp = PftParams::from_type(PftType::TemperateBroadleafDeciduous);
        assert!(trop.max_height > temp.max_height);
    }

    #[test]
    fn boreal_most_cold_hardy() {
        let boreal = PftParams::from_type(PftType::BorealNeedleleafEvergreen);
        let tropical = PftParams::from_type(PftType::TropicalBroadleafEvergreen);
        assert!(boreal.frost_threshold < tropical.frost_threshold);
    }

    #[test]
    fn conifer_lower_sla() {
        let conifer = PftParams::from_type(PftType::TemperateNeedleleafEvergreen);
        let broadleaf = PftParams::from_type(PftType::TemperateBroadleafDeciduous);
        assert!(conifer.sla < broadleaf.sla, "needles have lower SLA");
    }

    #[test]
    fn grass_short_lived() {
        let grass = PftParams::from_type(PftType::C3Grass);
        let tree = PftParams::from_type(PftType::TemperateBroadleafDeciduous);
        assert!(grass.lifespan_years < tree.lifespan_years);
    }

    #[test]
    fn deciduous_full_leaf_turnover() {
        let d = PftParams::from_type(PftType::TemperateBroadleafDeciduous);
        assert_eq!(d.leaf_turnover, 1.0);
    }

    #[test]
    fn conifer_low_leaf_turnover() {
        let c = PftParams::from_type(PftType::BorealNeedleleafEvergreen);
        assert!(c.leaf_turnover < 0.3, "boreal needles retained for years");
    }

    #[test]
    fn is_woody_correct() {
        assert!(PftParams::from_type(PftType::TemperateBroadleafDeciduous).is_woody());
        assert!(!PftParams::from_type(PftType::C3Grass).is_woody());
    }

    #[test]
    fn is_conifer_correct() {
        assert!(PftParams::from_type(PftType::TemperateNeedleleafEvergreen).is_conifer());
        assert!(!PftParams::from_type(PftType::TemperateBroadleafDeciduous).is_conifer());
    }

    #[test]
    fn alloc_reproductive_positive() {
        for pft in [
            PftType::TropicalBroadleafEvergreen,
            PftType::TemperateBroadleafDeciduous,
            PftType::C3Grass,
        ] {
            let p = PftParams::from_type(pft);
            assert!(p.alloc_reproductive() >= 0.0, "{pft:?}");
        }
    }

    #[test]
    fn grass_more_root_allocation() {
        let grass = PftParams::from_type(PftType::C3Grass);
        assert!(grass.alloc_root >= grass.alloc_stem, "grass is root-heavy");
    }
}
