//! Plant respiration — maintenance and growth respiration for carbon budget.
//!
//! Plants consume a significant fraction (40–60%) of gross photosynthesis
//! for their own metabolic needs. Net primary productivity (NPP) is what
//! remains: NPP = GPP - Ra (autotrophic respiration).
//!
//! Two components:
//! - **Maintenance respiration** — cost of keeping existing tissue alive
//! - **Growth respiration** — cost of constructing new tissue

use serde::{Deserialize, Serialize};

/// Respiration component type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RespirationComponent {
    /// Maintenance — metabolic cost of existing tissue.
    Maintenance,
    /// Growth — construction cost of new tissue.
    Growth,
}

/// Maintenance respiration rate (kg C/day).
///
/// Proportional to tissue nitrogen content and temperature (Ryan 1991).
///
/// `R_m = biomass × N_concentration × r_m_coeff × Q10^((T - 20) / 10)`
///
/// where r_m_coeff = 0.025 kg C / kg N / day at 20°C (Ryan 1991).
///
/// Maintenance respiration increases with:
/// - More biomass (more tissue to maintain)
/// - Higher tissue N (metabolically active tissue costs more)
/// - Higher temperature (enzyme kinetics)
///
/// Below 0°C, maintenance respiration is reduced to 10% (dormant metabolism).
///
/// - `biomass_kg` — total live biomass (kg dry matter)
/// - `nitrogen_concentration` — tissue N (kg N / kg biomass, typically 0.005–0.020)
/// - `temp_celsius` — air/tissue temperature (°C)
#[must_use]
pub fn maintenance_respiration(
    biomass_kg: f32,
    nitrogen_concentration: f32,
    temp_celsius: f32,
) -> f32 {
    if biomass_kg <= 0.0 || nitrogen_concentration <= 0.0 {
        return 0.0;
    }
    let r_m_coeff = 0.025_f32; // kg C / kg N / day at 20°C (Ryan 1991)
    let q10 = 2.0_f32;

    let temp_factor = if temp_celsius < 0.0 {
        0.1 // dormant — minimal respiration
    } else {
        q10.powf((temp_celsius - 20.0) / 10.0)
    };

    let r_m = biomass_kg * nitrogen_concentration * r_m_coeff * temp_factor;
    tracing::trace!(
        biomass_kg,
        nitrogen_concentration,
        temp_celsius,
        temp_factor,
        r_m,
        "maintenance_respiration"
    );
    r_m
}

/// Growth respiration (kg C consumed per kg C of new tissue).
///
/// Construction cost of converting photosynthate into structural biomass.
/// Approximately 25% of carbon allocated to growth is respired (Penning de
/// Vries 1975), meaning growth efficiency is ~75%.
///
/// `R_g = new_biomass_c × growth_respiration_fraction`
///
/// The fraction varies slightly by tissue type but 0.25 is a robust default.
///
/// - `new_biomass_carbon_kg` — carbon allocated to new growth (kg C)
#[must_use]
#[inline]
pub fn growth_respiration(new_biomass_carbon_kg: f32) -> f32 {
    if new_biomass_carbon_kg <= 0.0 {
        return 0.0;
    }
    let fraction = 0.25; // 25% of new C is respired (Penning de Vries 1975)
    let r_g = new_biomass_carbon_kg * fraction;
    tracing::trace!(new_biomass_carbon_kg, r_g, "growth_respiration");
    r_g
}

/// Growth respiration fraction — the proportion of allocated carbon lost
/// to construction costs (dimensionless, 0.0–1.0).
///
/// Default: 0.25 (Penning de Vries 1975).
/// Lipid-rich tissue (seeds): ~0.35. Cellulose: ~0.20.
///
/// - `is_reproductive` — true for seeds/fruits (higher lipid content)
#[must_use]
#[inline]
pub fn growth_respiration_fraction(is_reproductive: bool) -> f32 {
    if is_reproductive {
        0.35 // lipid-rich seeds cost more
    } else {
        0.25 // structural tissue (Penning de Vries 1975)
    }
}

/// Total autotrophic respiration (kg C/day).
///
/// `Ra = R_maintenance + R_growth`
///
/// - `maintenance_kg_c` — daily maintenance respiration (kg C/day)
/// - `growth_kg_c` — daily growth respiration (kg C/day)
#[must_use]
#[inline]
pub fn total_autotrophic_respiration(maintenance_kg_c: f32, growth_kg_c: f32) -> f32 {
    let ra = maintenance_kg_c.max(0.0) + growth_kg_c.max(0.0);
    tracing::trace!(
        maintenance_kg_c,
        growth_kg_c,
        ra,
        "total_autotrophic_respiration"
    );
    ra
}

/// Net primary productivity from gross photosynthesis (kg C/day).
///
/// `NPP = GPP - Ra`
///
/// This is the fundamental carbon budget equation. NPP is the carbon
/// available for growth, reproduction, and storage after metabolic costs.
///
/// If Ra > GPP (carbon starvation), NPP is negative — the plant is
/// consuming its reserves.
///
/// - `gross_photosynthesis_kg_c` — daily GPP (kg C/day)
/// - `autotrophic_respiration_kg_c` — daily Ra (kg C/day)
#[must_use]
#[inline]
pub fn net_primary_productivity_carbon(
    gross_photosynthesis_kg_c: f32,
    autotrophic_respiration_kg_c: f32,
) -> f32 {
    let npp = gross_photosynthesis_kg_c - autotrophic_respiration_kg_c;
    tracing::trace!(
        gross_photosynthesis_kg_c,
        autotrophic_respiration_kg_c,
        npp,
        "net_primary_productivity_carbon"
    );
    npp
}

/// Organ-specific maintenance respiration coefficients.
///
/// Different tissues have different metabolic costs. Leaves and fine roots
/// are metabolically expensive; wood is cheap (mostly dead cells).
///
/// Returns respiration coefficient (kg C / kg N / day at 20°C).
///
/// - `is_woody` — true for stem/coarse root tissue
#[must_use]
#[inline]
pub fn organ_respiration_coefficient(is_woody: bool) -> f32 {
    if is_woody {
        0.010 // wood: low metabolic activity (sapwood only)
    } else {
        0.025 // leaves/fine roots: high metabolic activity (Ryan 1991)
    }
}

/// Partitioned maintenance respiration for a plant with known organ biomass (kg C/day).
///
/// Computes maintenance respiration separately for each organ pool,
/// using organ-specific coefficients and a shared temperature factor.
///
/// - `leaf_kg` — leaf biomass (kg)
/// - `stem_kg` — stem biomass (kg)
/// - `root_kg` — root biomass (kg)
/// - `leaf_n` — leaf N concentration (kg N / kg biomass)
/// - `wood_n` — stem/root N concentration (kg N / kg biomass)
/// - `temp_celsius` — temperature (°C)
#[must_use]
pub fn partitioned_maintenance_respiration(
    leaf_kg: f32,
    stem_kg: f32,
    root_kg: f32,
    leaf_n: f32,
    wood_n: f32,
    temp_celsius: f32,
) -> f32 {
    let q10 = 2.0_f32;
    let temp_factor = if temp_celsius < 0.0 {
        0.1
    } else {
        q10.powf((temp_celsius - 20.0) / 10.0)
    };

    let leaf_coeff = 0.025; // high metabolic activity
    let wood_coeff = 0.010; // low metabolic activity

    let r_leaf = leaf_kg.max(0.0) * leaf_n.max(0.0) * leaf_coeff * temp_factor;
    let r_stem = stem_kg.max(0.0) * wood_n.max(0.0) * wood_coeff * temp_factor;
    let r_root = root_kg.max(0.0) * wood_n.max(0.0) * wood_coeff * temp_factor;
    let total = r_leaf + r_stem + r_root;

    tracing::trace!(
        leaf_kg,
        stem_kg,
        root_kg,
        temp_celsius,
        r_leaf,
        r_stem,
        r_root,
        total,
        "partitioned_maintenance_respiration"
    );
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- maintenance respiration ---

    #[test]
    fn maintenance_at_reference_temp() {
        // 100 kg biomass, 1.2% N, 20°C → 100 × 0.012 × 0.025 × 1.0 = 0.03 kg C/day
        let r = maintenance_respiration(100.0, 0.012, 20.0);
        assert!((r - 0.03).abs() < 0.001, "got {r}");
    }

    #[test]
    fn maintenance_doubles_at_30c() {
        let r20 = maintenance_respiration(100.0, 0.012, 20.0);
        let r30 = maintenance_respiration(100.0, 0.012, 30.0);
        assert!((r30 / r20 - 2.0).abs() < 0.01, "Q10=2 should double");
    }

    #[test]
    fn maintenance_frozen_reduced() {
        let warm = maintenance_respiration(100.0, 0.012, 20.0);
        let frozen = maintenance_respiration(100.0, 0.012, -5.0);
        assert!(frozen < warm * 0.15, "frozen should be ~10% of reference");
    }

    #[test]
    fn maintenance_zero_biomass() {
        assert_eq!(maintenance_respiration(0.0, 0.012, 20.0), 0.0);
    }

    #[test]
    fn maintenance_zero_nitrogen() {
        assert_eq!(maintenance_respiration(100.0, 0.0, 20.0), 0.0);
    }

    #[test]
    fn maintenance_proportional_to_biomass() {
        let small = maintenance_respiration(50.0, 0.012, 20.0);
        let large = maintenance_respiration(100.0, 0.012, 20.0);
        assert!((large / small - 2.0).abs() < 0.01);
    }

    #[test]
    fn maintenance_proportional_to_nitrogen() {
        let low_n = maintenance_respiration(100.0, 0.006, 20.0);
        let high_n = maintenance_respiration(100.0, 0.012, 20.0);
        assert!((high_n / low_n - 2.0).abs() < 0.01);
    }

    // --- growth respiration ---

    #[test]
    fn growth_respiration_basic() {
        let r = growth_respiration(1.0);
        assert!((r - 0.25).abs() < 0.01, "25% of new C, got {r}");
    }

    #[test]
    fn growth_respiration_zero() {
        assert_eq!(growth_respiration(0.0), 0.0);
    }

    #[test]
    fn growth_respiration_negative() {
        assert_eq!(growth_respiration(-1.0), 0.0);
    }

    #[test]
    fn growth_fraction_reproductive_higher() {
        assert!(growth_respiration_fraction(true) > growth_respiration_fraction(false));
    }

    // --- total autotrophic ---

    #[test]
    fn total_respiration_sums() {
        let ra = total_autotrophic_respiration(0.03, 0.01);
        assert!((ra - 0.04).abs() < 0.001);
    }

    // --- NPP ---

    #[test]
    fn npp_positive_when_gpp_exceeds_ra() {
        let npp = net_primary_productivity_carbon(0.10, 0.04);
        assert!((npp - 0.06).abs() < 0.001);
    }

    #[test]
    fn npp_negative_carbon_starvation() {
        let npp = net_primary_productivity_carbon(0.02, 0.04);
        assert!(npp < 0.0, "Ra > GPP should cause negative NPP");
    }

    #[test]
    fn npp_zero_when_balanced() {
        let npp = net_primary_productivity_carbon(0.04, 0.04);
        assert!((npp).abs() < 0.001);
    }

    // --- organ-specific ---

    #[test]
    fn wood_cheaper_than_leaves() {
        assert!(organ_respiration_coefficient(true) < organ_respiration_coefficient(false));
    }

    // --- partitioned ---

    #[test]
    fn partitioned_leaf_dominant() {
        // Equal biomass but leaves are metabolically expensive
        let r = partitioned_maintenance_respiration(50.0, 50.0, 50.0, 0.020, 0.005, 20.0);
        // Leaf: 50 × 0.020 × 0.025 = 0.025
        // Stem: 50 × 0.005 × 0.010 = 0.0025
        // Root: 50 × 0.005 × 0.010 = 0.0025
        // Total: 0.030
        assert!((r - 0.030).abs() < 0.001, "got {r}");
    }

    #[test]
    fn partitioned_zero_biomass() {
        let r = partitioned_maintenance_respiration(0.0, 0.0, 0.0, 0.020, 0.005, 20.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn partitioned_temperature_response() {
        let cool = partitioned_maintenance_respiration(50.0, 100.0, 50.0, 0.015, 0.005, 10.0);
        let warm = partitioned_maintenance_respiration(50.0, 100.0, 50.0, 0.015, 0.005, 25.0);
        assert!(warm > cool);
    }
}
