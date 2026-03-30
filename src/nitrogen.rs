//! Soil nitrogen cycling — mineralization, plant uptake, leaching, and
//! nitrogen limitation on plant growth. Nitrogen is the most commonly
//! limiting nutrient in terrestrial ecosystems.

use serde::{Deserialize, Serialize};

/// Soil nitrogen pool state.
///
/// Tracks two pools:
/// - **Available N** (mineral: NH₄⁺ + NO₃⁻) — plant-accessible, mobile
/// - **Organic N** — bound in soil organic matter, slowly mineralized
///
/// All values in kg N/m² (equivalent to 10 × g N/m²).
/// Typical surface soil (0–30 cm):
/// - Available N: 0.001–0.01 kg/m² (10–100 g/m²)
/// - Organic N: 0.1–1.0 kg/m² (temperate forest ~0.5 kg/m²)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoilNitrogen {
    /// Plant-available mineral nitrogen (kg N/m²).
    pub available_n: f32,
    /// Organic nitrogen in soil organic matter (kg N/m²).
    pub organic_n: f32,
}

impl SoilNitrogen {
    /// Create a soil nitrogen pool.
    ///
    /// - `available_n` — mineral N (kg N/m²)
    /// - `organic_n` — organic N (kg N/m²)
    #[must_use]
    pub fn new(available_n: f32, organic_n: f32) -> Self {
        Self {
            available_n: available_n.max(0.0),
            organic_n: organic_n.max(0.0),
        }
    }

    /// Fertile agricultural soil: high available N, moderate organic pool.
    #[must_use]
    pub fn fertile() -> Self {
        Self {
            available_n: 0.008, // kg N/m² (~80 g/m²)
            organic_n: 0.4,     // kg N/m²
        }
    }

    /// Temperate forest soil: moderate available N, large organic pool.
    #[must_use]
    pub fn forest() -> Self {
        Self {
            available_n: 0.004, // kg N/m² (~40 g/m²)
            organic_n: 0.5,     // kg N/m²
        }
    }

    /// Poor sandy soil: low N in both pools.
    #[must_use]
    pub fn poor() -> Self {
        Self {
            available_n: 0.001, // kg N/m² (~10 g/m²)
            organic_n: 0.1,     // kg N/m²
        }
    }

    /// Total nitrogen in both pools (kg N/m²).
    #[must_use]
    #[inline]
    pub fn total_n(&self) -> f32 {
        self.available_n + self.organic_n
    }

    /// Add mineral nitrogen (fertilization, atmospheric deposition, fixation).
    /// Returns actual amount added (kg N/m²).
    ///
    /// - `n_kg_m2` — nitrogen to add (kg N/m²)
    pub fn add_available(&mut self, n_kg_m2: f32) -> f32 {
        if n_kg_m2 <= 0.0 {
            return 0.0;
        }
        self.available_n += n_kg_m2;
        tracing::trace!(n_kg_m2, available = self.available_n, "add_available");
        n_kg_m2
    }

    /// Add organic nitrogen (litter decomposition input to organic pool).
    /// Returns actual amount added (kg N/m²).
    ///
    /// - `n_kg_m2` — nitrogen to add (kg N/m²)
    pub fn add_organic(&mut self, n_kg_m2: f32) -> f32 {
        if n_kg_m2 <= 0.0 {
            return 0.0;
        }
        self.organic_n += n_kg_m2;
        tracing::trace!(n_kg_m2, organic = self.organic_n, "add_organic");
        n_kg_m2
    }

    /// Remove available nitrogen (plant uptake, leaching). Returns actual removed.
    /// Cannot go below zero.
    ///
    /// - `n_kg_m2` — nitrogen to remove (kg N/m²)
    pub fn remove_available(&mut self, n_kg_m2: f32) -> f32 {
        if n_kg_m2 <= 0.0 {
            return 0.0;
        }
        let actual = n_kg_m2.min(self.available_n);
        self.available_n -= actual;
        tracing::trace!(
            requested = n_kg_m2,
            actual,
            available = self.available_n,
            "remove_available"
        );
        actual
    }
}

/// Net mineralization rate (kg N/m²/day).
///
/// Organic N → available N, modulated by temperature and moisture.
///
/// `rate = organic_n × k_min × temp_factor × moisture_factor`
///
/// Base annual mineralization rate k_min = 0.02/yr (2% of organic N per year,
/// typical for temperate soils — Schimel & Bennett 2004).
///
/// Temperature factor: Q10 = 2.0, reference 25°C (same as decomposition).
/// Moisture factor: bell curve, optimum at 0.6 (field capacity).
///
/// - `organic_n` — organic N pool (kg N/m²)
/// - `temp_celsius` — soil temperature (°C)
/// - `moisture_fraction` — soil moisture (0.0–1.0)
#[must_use]
pub fn mineralization_rate(organic_n: f32, temp_celsius: f32, moisture_fraction: f32) -> f32 {
    if organic_n <= 0.0 || temp_celsius < 0.0 {
        return 0.0;
    }
    let k_annual = 0.02; // 2% of organic N per year
    let k_daily = k_annual / 365.0;

    // Q10 temperature factor (same as decomposition)
    let q10 = 2.0_f32;
    let temp_f = q10.powf((temp_celsius - 25.0) / 10.0);

    // Moisture bell curve (same shape as decomposition)
    let m = moisture_fraction.clamp(0.0, 1.0);
    let diff = m - 0.6;
    let moist_f = (-diff * diff / 0.08).exp();

    let rate = organic_n * k_daily * temp_f * moist_f;
    tracing::trace!(
        organic_n,
        temp_celsius,
        moisture_fraction,
        temp_f,
        moist_f,
        rate,
        "mineralization_rate"
    );
    rate
}

/// Plant nitrogen uptake (kg N/m²/day).
///
/// Actual uptake is the minimum of:
/// 1. Plant demand (based on growth rate and tissue N concentration)
/// 2. Available soil N
/// 3. Root-limited uptake capacity
///
/// Root capacity scales with root biomass: `capacity = root_kg × 0.001 × moisture_factor`
/// (empirical: 1 kg roots can extract ~0.001 kg N/day at optimal moisture).
///
/// Moisture factor: linear ramp from 0 at wilting to 1 at field capacity,
/// because N moves to roots via mass flow in soil solution.
///
/// - `demand_kg_m2` — plant N demand (kg N/m²/day)
/// - `available_n` — available mineral N in soil (kg N/m²)
/// - `root_biomass_kg` — root dry mass (kg)
/// - `soil_moisture_fraction` — relative soil water content (0.0–1.0)
#[must_use]
pub fn nitrogen_uptake(
    demand_kg_m2: f32,
    available_n: f32,
    root_biomass_kg: f32,
    soil_moisture_fraction: f32,
) -> f32 {
    if demand_kg_m2 <= 0.0 || available_n <= 0.0 || root_biomass_kg <= 0.0 {
        return 0.0;
    }
    // Root capacity: 0.001 kg N per kg root per day at optimal moisture
    let moisture_f = soil_moisture_fraction.clamp(0.0, 1.0);
    let root_capacity = root_biomass_kg * 0.001 * moisture_f;

    let uptake = demand_kg_m2.min(available_n).min(root_capacity);
    tracing::trace!(
        demand_kg_m2,
        available_n,
        root_biomass_kg,
        moisture_f,
        root_capacity,
        uptake,
        "nitrogen_uptake"
    );
    uptake.max(0.0)
}

/// Nitrogen leaching loss (kg N/m²/day).
///
/// Nitrate is mobile and leaches with drainage water.
///
/// `leached = available_n × leaching_fraction`
///
/// Leaching fraction = `drainage_mm / (soil_water_mm + drainage_mm)` — the
/// proportion of soil solution that drains away (Burns 1980 mixing model).
///
/// - `available_n` — available mineral N (kg N/m²)
/// - `drainage_mm` — daily drainage below root zone (mm)
/// - `soil_water_mm` — current soil water content (mm)
#[must_use]
pub fn nitrogen_leaching(available_n: f32, drainage_mm: f32, soil_water_mm: f32) -> f32 {
    if available_n <= 0.0 || drainage_mm <= 0.0 {
        return 0.0;
    }
    let total_water = soil_water_mm.max(0.0) + drainage_mm;
    if total_water <= 0.0 {
        return 0.0;
    }
    let leach_fraction = drainage_mm / total_water;
    let leached = available_n * leach_fraction;
    tracing::trace!(
        available_n,
        drainage_mm,
        soil_water_mm,
        leach_fraction,
        leached,
        "nitrogen_leaching"
    );
    leached
}

/// Nitrogen stress factor on growth (0.0–1.0).
///
/// Plants require a minimum nitrogen concentration in tissue for growth.
/// Below the critical N content, growth declines linearly.
///
/// `factor = min(1.0, plant_n / critical_n)`
///
/// Critical N concentration varies by species:
/// - Broadleaf trees: ~0.012 kg N / kg biomass (1.2%)
/// - Conifers: ~0.008 kg N / kg biomass (0.8%)
/// - Grasses: ~0.015 kg N / kg biomass (1.5%)
/// - Crops: ~0.020 kg N / kg biomass (2.0%)
///
/// Based on Ågren (1985) critical N dilution concept.
///
/// - `plant_n_concentration` — current tissue N (kg N / kg biomass)
/// - `critical_n_concentration` — minimum N for unstressed growth (kg N / kg biomass)
#[must_use]
#[inline]
pub fn nitrogen_stress_factor(plant_n_concentration: f32, critical_n_concentration: f32) -> f32 {
    if plant_n_concentration <= 0.0 || critical_n_concentration <= 0.0 {
        return 0.0;
    }
    let factor = (plant_n_concentration / critical_n_concentration).clamp(0.0, 1.0);
    tracing::trace!(
        plant_n_concentration,
        critical_n_concentration,
        factor,
        "nitrogen_stress_factor"
    );
    factor
}

/// Critical nitrogen concentration for common plant types (kg N / kg biomass).
///
/// Returns the minimum tissue N concentration needed for unstressed growth.
///
/// - `is_conifer` — true for conifers (lower N requirement)
#[must_use]
#[inline]
pub fn critical_n_concentration(is_conifer: bool) -> f32 {
    if is_conifer {
        0.008 // 0.8% — conifers are N-efficient
    } else {
        0.012 // 1.2% — broadleaf/deciduous
    }
}

/// Plant nitrogen demand (kg N/m²/day).
///
/// New growth requires nitrogen in proportion to biomass gain.
///
/// `demand = growth_rate_kg_m2_day × target_n_concentration`
///
/// Target N is typically 1.5× the critical concentration (luxury uptake).
///
/// - `growth_rate_kg_m2_day` — daily biomass gain (kg/m²/day)
/// - `target_n_concentration` — desired tissue N (kg N / kg biomass)
#[must_use]
#[inline]
pub fn plant_n_demand(growth_rate_kg_m2_day: f32, target_n_concentration: f32) -> f32 {
    if growth_rate_kg_m2_day <= 0.0 || target_n_concentration <= 0.0 {
        return 0.0;
    }
    let demand = growth_rate_kg_m2_day * target_n_concentration;
    tracing::trace!(
        growth_rate_kg_m2_day,
        target_n_concentration,
        demand,
        "plant_n_demand"
    );
    demand
}

/// Daily nitrogen flux summary (all values in kg N/m²).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NitrogenFluxes {
    /// Organic N mineralized to available pool (kg N/m²).
    pub mineralization: f32,
    /// N taken up by plants (kg N/m²).
    pub plant_uptake: f32,
    /// N lost to leaching below root zone (kg N/m²).
    pub leaching: f32,
}

impl NitrogenFluxes {
    /// Net change in available N pool (kg N/m²).
    /// Positive = gaining available N, negative = losing.
    #[must_use]
    #[inline]
    pub fn net_available_change(&self) -> f32 {
        self.mineralization - self.plant_uptake - self.leaching
    }
}

/// Run a daily nitrogen balance step on the soil pool.
///
/// Processes in order:
/// 1. Mineralization: organic N → available N
/// 2. Plant uptake: available N → plant
/// 3. Leaching: available N lost with drainage water
///
/// Mutates `soil_n` in place and returns the flux summary.
///
/// - `soil_n` — mutable soil nitrogen state
/// - `temp_celsius` — soil temperature (°C)
/// - `moisture_fraction` — relative soil water content (0.0–1.0)
/// - `plant_demand_kg_m2` — plant N demand (kg N/m²/day)
/// - `root_biomass_kg` — root dry mass (kg)
/// - `drainage_mm` — daily drainage (mm, from water balance)
/// - `soil_water_mm` — current soil water (mm)
pub fn daily_nitrogen_balance(
    soil_n: &mut SoilNitrogen,
    temp_celsius: f32,
    moisture_fraction: f32,
    plant_demand_kg_m2: f32,
    root_biomass_kg: f32,
    drainage_mm: f32,
    soil_water_mm: f32,
) -> NitrogenFluxes {
    // 1. Mineralization: organic → available
    let min_rate = mineralization_rate(soil_n.organic_n, temp_celsius, moisture_fraction);
    let mineralized = min_rate.min(soil_n.organic_n);
    soil_n.organic_n -= mineralized;
    soil_n.available_n += mineralized;

    // 2. Plant uptake
    let uptake = nitrogen_uptake(
        plant_demand_kg_m2,
        soil_n.available_n,
        root_biomass_kg,
        moisture_fraction,
    );
    soil_n.remove_available(uptake);

    // 3. Leaching
    let leached = nitrogen_leaching(soil_n.available_n, drainage_mm, soil_water_mm);
    soil_n.remove_available(leached);

    let fluxes = NitrogenFluxes {
        mineralization: mineralized,
        plant_uptake: uptake,
        leaching: leached,
    };

    tracing::trace!(
        mineralization = fluxes.mineralization,
        plant_uptake = fluxes.plant_uptake,
        leaching = fluxes.leaching,
        net = fluxes.net_available_change(),
        available = soil_n.available_n,
        organic = soil_n.organic_n,
        "daily_nitrogen_balance"
    );

    fluxes
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SoilNitrogen construction ---

    #[test]
    fn fertile_has_more_available_than_poor() {
        assert!(SoilNitrogen::fertile().available_n > SoilNitrogen::poor().available_n);
    }

    #[test]
    fn forest_has_large_organic_pool() {
        let f = SoilNitrogen::forest();
        assert!(f.organic_n > f.available_n);
    }

    #[test]
    fn total_n_sums_pools() {
        let sn = SoilNitrogen::new(0.005, 0.3);
        assert!((sn.total_n() - 0.305).abs() < 0.001);
    }

    #[test]
    fn new_clamps_negative() {
        let sn = SoilNitrogen::new(-1.0, -2.0);
        assert_eq!(sn.available_n, 0.0);
        assert_eq!(sn.organic_n, 0.0);
    }

    // --- add/remove ---

    #[test]
    fn add_available_increases() {
        let mut sn = SoilNitrogen::poor();
        let before = sn.available_n;
        sn.add_available(0.002);
        assert!((sn.available_n - before - 0.002).abs() < 0.0001);
    }

    #[test]
    fn add_available_negative_noop() {
        let mut sn = SoilNitrogen::poor();
        let before = sn.available_n;
        assert_eq!(sn.add_available(-0.001), 0.0);
        assert_eq!(sn.available_n, before);
    }

    #[test]
    fn add_organic_increases() {
        let mut sn = SoilNitrogen::poor();
        let before = sn.organic_n;
        sn.add_organic(0.01);
        assert!((sn.organic_n - before - 0.01).abs() < 0.001);
    }

    #[test]
    fn remove_available_basic() {
        let mut sn = SoilNitrogen::fertile();
        let removed = sn.remove_available(0.002);
        assert!((removed - 0.002).abs() < 0.0001);
    }

    #[test]
    fn remove_available_cant_go_negative() {
        let mut sn = SoilNitrogen::new(0.001, 0.1);
        let removed = sn.remove_available(0.01);
        assert!((removed - 0.001).abs() < 0.0001);
        assert_eq!(sn.available_n, 0.0);
    }

    #[test]
    fn remove_available_negative_noop() {
        let mut sn = SoilNitrogen::fertile();
        assert_eq!(sn.remove_available(-0.001), 0.0);
    }

    // --- mineralization ---

    #[test]
    fn mineralization_at_reference_conditions() {
        // 0.5 kg organic N, 25°C, optimal moisture (0.6)
        // rate = 0.5 × (0.02/365) × 1.0 × 1.0 ≈ 0.0000274
        let rate = mineralization_rate(0.5, 25.0, 0.6);
        let expected = 0.5 * 0.02 / 365.0;
        assert!(
            (rate - expected).abs() < 0.00001,
            "got {rate}, expected {expected}"
        );
    }

    #[test]
    fn mineralization_frozen_is_zero() {
        assert_eq!(mineralization_rate(0.5, -5.0, 0.6), 0.0);
    }

    #[test]
    fn mineralization_no_organic_is_zero() {
        assert_eq!(mineralization_rate(0.0, 25.0, 0.6), 0.0);
    }

    #[test]
    fn mineralization_warmer_is_faster() {
        let cool = mineralization_rate(0.5, 15.0, 0.6);
        let warm = mineralization_rate(0.5, 30.0, 0.6);
        assert!(warm > cool);
    }

    #[test]
    fn mineralization_dry_is_slower() {
        let moist = mineralization_rate(0.5, 25.0, 0.6);
        let dry = mineralization_rate(0.5, 25.0, 0.1);
        assert!(moist > dry);
    }

    #[test]
    fn mineralization_proportional_to_organic_n() {
        let low = mineralization_rate(0.1, 25.0, 0.6);
        let high = mineralization_rate(0.5, 25.0, 0.6);
        assert!((high / low - 5.0).abs() < 0.01);
    }

    // --- nitrogen uptake ---

    #[test]
    fn uptake_limited_by_demand() {
        let uptake = nitrogen_uptake(0.0001, 0.01, 500.0, 1.0);
        assert!((uptake - 0.0001).abs() < 0.00001, "got {uptake}");
    }

    #[test]
    fn uptake_limited_by_available() {
        let uptake = nitrogen_uptake(0.1, 0.0005, 500.0, 1.0);
        assert!((uptake - 0.0005).abs() < 0.00001);
    }

    #[test]
    fn uptake_limited_by_root_capacity() {
        // 10 kg roots × 0.001 × 1.0 = 0.01 capacity
        let uptake = nitrogen_uptake(0.1, 0.1, 10.0, 1.0);
        assert!((uptake - 0.01).abs() < 0.001, "got {uptake}");
    }

    #[test]
    fn uptake_zero_demand() {
        assert_eq!(nitrogen_uptake(0.0, 0.01, 500.0, 1.0), 0.0);
    }

    #[test]
    fn uptake_zero_roots() {
        assert_eq!(nitrogen_uptake(0.001, 0.01, 0.0, 1.0), 0.0);
    }

    #[test]
    fn uptake_zero_moisture() {
        assert_eq!(nitrogen_uptake(0.001, 0.01, 500.0, 0.0), 0.0);
    }

    #[test]
    fn uptake_dry_soil_reduces() {
        // Use values where root capacity is the binding constraint
        // 5 kg roots: wet capacity = 0.005, dry capacity = 0.0015
        let wet = nitrogen_uptake(0.1, 0.1, 5.0, 1.0);
        let dry = nitrogen_uptake(0.1, 0.1, 5.0, 0.3);
        assert!(
            dry < wet,
            "dry soil should reduce N uptake: wet={wet}, dry={dry}"
        );
    }

    #[test]
    fn uptake_more_roots_more_uptake() {
        // Small roots limited
        let small = nitrogen_uptake(0.1, 0.1, 1.0, 1.0); // capacity = 0.001
        let large = nitrogen_uptake(0.1, 0.1, 100.0, 1.0); // capacity = 0.1
        assert!(large > small);
    }

    // --- leaching ---

    #[test]
    fn leaching_with_drainage() {
        // 0.01 available, 50mm drainage, 200mm soil water
        // fraction = 50/(200+50) = 0.2, leached = 0.01 × 0.2 = 0.002
        let leached = nitrogen_leaching(0.01, 50.0, 200.0);
        assert!((leached - 0.002).abs() < 0.0001, "got {leached}");
    }

    #[test]
    fn leaching_no_drainage() {
        assert_eq!(nitrogen_leaching(0.01, 0.0, 200.0), 0.0);
    }

    #[test]
    fn leaching_no_nitrogen() {
        assert_eq!(nitrogen_leaching(0.0, 50.0, 200.0), 0.0);
    }

    #[test]
    fn leaching_more_drainage_more_loss() {
        let low = nitrogen_leaching(0.01, 10.0, 200.0);
        let high = nitrogen_leaching(0.01, 100.0, 200.0);
        assert!(high > low);
    }

    #[test]
    fn leaching_proportional_to_available() {
        let low = nitrogen_leaching(0.005, 50.0, 200.0);
        let high = nitrogen_leaching(0.010, 50.0, 200.0);
        assert!((high / low - 2.0).abs() < 0.01);
    }

    // --- nitrogen stress factor ---

    #[test]
    fn stress_factor_sufficient_n() {
        assert_eq!(nitrogen_stress_factor(0.015, 0.012), 1.0);
    }

    #[test]
    fn stress_factor_at_critical() {
        let f = nitrogen_stress_factor(0.012, 0.012);
        assert!((f - 1.0).abs() < 0.01);
    }

    #[test]
    fn stress_factor_half_critical() {
        let f = nitrogen_stress_factor(0.006, 0.012);
        assert!((f - 0.5).abs() < 0.01, "got {f}");
    }

    #[test]
    fn stress_factor_zero_n() {
        assert_eq!(nitrogen_stress_factor(0.0, 0.012), 0.0);
    }

    #[test]
    fn stress_factor_negative() {
        assert_eq!(nitrogen_stress_factor(-0.001, 0.012), 0.0);
    }

    #[test]
    fn stress_factor_zero_critical() {
        assert_eq!(nitrogen_stress_factor(0.01, 0.0), 0.0);
    }

    #[test]
    fn stress_factor_clamped_at_one() {
        // luxury N (above critical) should still return 1.0
        assert_eq!(nitrogen_stress_factor(0.020, 0.012), 1.0);
    }

    // --- critical N ---

    #[test]
    fn conifer_lower_critical_n() {
        assert!(critical_n_concentration(true) < critical_n_concentration(false));
    }

    #[test]
    fn critical_n_positive() {
        assert!(critical_n_concentration(true) > 0.0);
        assert!(critical_n_concentration(false) > 0.0);
    }

    // --- plant demand ---

    #[test]
    fn demand_proportional_to_growth() {
        let low = plant_n_demand(0.001, 0.012);
        let high = plant_n_demand(0.005, 0.012);
        assert!((high / low - 5.0).abs() < 0.01);
    }

    #[test]
    fn demand_zero_growth() {
        assert_eq!(plant_n_demand(0.0, 0.012), 0.0);
    }

    #[test]
    fn demand_zero_target() {
        assert_eq!(plant_n_demand(0.001, 0.0), 0.0);
    }

    // --- NitrogenFluxes ---

    #[test]
    fn fluxes_net_change() {
        let f = NitrogenFluxes {
            mineralization: 0.001,
            plant_uptake: 0.0005,
            leaching: 0.0002,
        };
        let net = f.net_available_change();
        assert!((net - 0.0003).abs() < 0.00001);
    }

    // --- daily balance ---

    #[test]
    fn balance_mineralization_transfers() {
        let mut sn = SoilNitrogen::forest();
        let before_available = sn.available_n;
        let before_organic = sn.organic_n;
        let f = daily_nitrogen_balance(&mut sn, 25.0, 0.6, 0.0, 0.0, 0.0, 200.0);
        assert!(f.mineralization > 0.0, "should mineralize at warm temps");
        assert!(
            sn.available_n > before_available,
            "available should increase"
        );
        assert!(sn.organic_n < before_organic, "organic should decrease");
    }

    #[test]
    fn balance_uptake_removes() {
        let mut sn = SoilNitrogen::fertile();
        let before = sn.available_n;
        let f = daily_nitrogen_balance(&mut sn, 25.0, 0.8, 0.001, 500.0, 0.0, 200.0);
        assert!(f.plant_uptake > 0.0);
        // Available may be higher or lower due to mineralization + uptake
        assert!(
            sn.available_n < before + f.mineralization,
            "uptake should reduce available"
        );
    }

    #[test]
    fn balance_leaching_with_drainage() {
        let mut sn = SoilNitrogen::fertile();
        let f = daily_nitrogen_balance(&mut sn, 25.0, 0.6, 0.0, 0.0, 50.0, 200.0);
        assert!(f.leaching > 0.0, "drainage should leach N");
    }

    #[test]
    fn balance_frozen_no_mineralization() {
        let mut sn = SoilNitrogen::forest();
        let f = daily_nitrogen_balance(&mut sn, -5.0, 0.6, 0.0, 0.0, 0.0, 200.0);
        assert_eq!(f.mineralization, 0.0);
    }

    #[test]
    fn balance_conservation() {
        let mut sn = SoilNitrogen::forest();
        let before_total = sn.total_n();
        let f = daily_nitrogen_balance(&mut sn, 25.0, 0.6, 0.0005, 200.0, 10.0, 200.0);
        let after_total = sn.total_n();
        // Total change should equal -(uptake + leaching)
        let expected_loss = f.plant_uptake + f.leaching;
        let actual_loss = before_total - after_total;
        assert!(
            (actual_loss - expected_loss).abs() < 0.000001,
            "N balance: lost={actual_loss}, expected={expected_loss}"
        );
    }

    #[test]
    fn balance_multi_day_depletion() {
        let mut sn = SoilNitrogen::poor();
        // Heavy demand, no replenishment
        for _ in 0..365 {
            daily_nitrogen_balance(&mut sn, 20.0, 0.5, 0.001, 100.0, 5.0, 200.0);
        }
        assert!(
            sn.available_n < SoilNitrogen::poor().available_n * 0.1,
            "year of heavy demand should deplete available N, got {}",
            sn.available_n
        );
    }

    #[test]
    fn balance_no_inputs_no_outputs() {
        let mut sn = SoilNitrogen::new(0.0, 0.0);
        let f = daily_nitrogen_balance(&mut sn, 25.0, 0.6, 0.001, 100.0, 10.0, 200.0);
        assert_eq!(f.mineralization, 0.0);
        assert_eq!(f.plant_uptake, 0.0);
        assert_eq!(f.leaching, 0.0);
    }
}
