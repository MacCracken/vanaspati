use crate::water::SoilWater;
use serde::{Deserialize, Serialize};

/// Root system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RootType {
    Taproot,
    Fibrous,
    Adventitious,
}

/// Root system properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootSystem {
    pub root_type: RootType,
    pub max_depth_m: f32,
    pub spread_radius_m: f32,
    pub water_uptake_rate: f32, // liters/day
}

impl RootSystem {
    #[must_use]
    pub fn oak() -> Self {
        Self {
            root_type: RootType::Taproot,
            max_depth_m: 5.0,
            spread_radius_m: 15.0,
            water_uptake_rate: 200.0,
        }
    }
    #[must_use]
    pub fn grass() -> Self {
        Self {
            root_type: RootType::Fibrous,
            max_depth_m: 0.3,
            spread_radius_m: 0.2,
            water_uptake_rate: 0.5,
        }
    }

    /// Mangrove: adventitious prop roots, coastal.
    #[must_use]
    pub fn mangrove() -> Self {
        Self {
            root_type: RootType::Adventitious,
            max_depth_m: 1.5,
            spread_radius_m: 5.0,
            water_uptake_rate: 50.0,
        }
    }

    /// Root zone fraction — how much of the soil column this root system can access (0.0–1.0).
    ///
    /// `fraction = min(1.0, max_depth_m / soil_depth_m)`
    ///
    /// Shallow roots in deep soil access less water; deep roots in shallow soil access all.
    ///
    /// - `soil_depth_m` — soil column depth (meters)
    #[must_use]
    #[inline]
    pub fn root_zone_fraction(&self, soil_depth_m: f32) -> f32 {
        if soil_depth_m <= 0.0 {
            return 0.0;
        }
        let fraction = (self.max_depth_m / soil_depth_m).clamp(0.0, 1.0);
        tracing::trace!(
            max_depth_m = self.max_depth_m,
            soil_depth_m,
            fraction,
            "root_zone_fraction"
        );
        fraction
    }

    /// Water uptake from soil (mm/day).
    ///
    /// Actual uptake is the minimum of:
    /// 1. Transpiration demand (what the canopy needs)
    /// 2. Root capacity (`water_uptake_rate` scaled to soil area, liters/day → mm/day)
    /// 3. Available water in the root zone
    ///
    /// Root zone depth limits access: shallow roots in deep soil only see a
    /// fraction of available water.
    ///
    /// - `soil` — current soil water state
    /// - `transpiration_demand_mm` — canopy transpiration demand (mm/day)
    #[must_use]
    pub fn water_uptake_mm(&self, soil: &SoilWater, transpiration_demand_mm: f32) -> f32 {
        if transpiration_demand_mm <= 0.0 {
            return 0.0;
        }
        let zone_frac = self.root_zone_fraction(soil.depth_m);
        // Available water in the root zone (mm)
        let zone_available = soil.available_water_mm() * zone_frac;
        // Root capacity: water_uptake_rate is liters/day; 1 liter/m² = 1 mm
        let root_capacity = self.water_uptake_rate;
        let uptake = transpiration_demand_mm
            .min(root_capacity)
            .min(zone_available);
        tracing::trace!(
            transpiration_demand_mm,
            zone_frac,
            zone_available,
            root_capacity = self.water_uptake_rate,
            uptake,
            "water_uptake_mm"
        );
        uptake.max(0.0)
    }

    /// Soil stabilization factor (0–1, dimensionless). Fibrous roots stabilize better per area.
    #[must_use]
    pub fn stabilization_factor(&self) -> f32 {
        let factor = match self.root_type {
            RootType::Fibrous => 0.9,
            RootType::Taproot => 0.6,
            RootType::Adventitious => 0.7,
        };
        tracing::trace!(?self.root_type, factor, "stabilization_factor");
        factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibrous_better_stabilization() {
        assert!(
            RootSystem::grass().stabilization_factor() > RootSystem::oak().stabilization_factor()
        );
    }

    #[test]
    fn oak_deeper_than_grass() {
        assert!(RootSystem::oak().max_depth_m > RootSystem::grass().max_depth_m);
    }

    #[test]
    fn oak_wider_spread() {
        assert!(RootSystem::oak().spread_radius_m > RootSystem::grass().spread_radius_m);
    }

    #[test]
    fn oak_higher_water_uptake() {
        assert!(RootSystem::oak().water_uptake_rate > RootSystem::grass().water_uptake_rate);
    }

    #[test]
    fn stabilization_in_valid_range() {
        for root in [RootSystem::oak(), RootSystem::grass()] {
            let s = root.stabilization_factor();
            assert!((0.0..=1.0).contains(&s), "stabilization must be 0-1");
        }
    }

    // --- root_zone_fraction tests ---

    #[test]
    fn root_zone_fraction_deep_roots_shallow_soil() {
        let oak = RootSystem::oak(); // 5m roots
        let frac = oak.root_zone_fraction(1.0); // 1m soil
        assert_eq!(frac, 1.0, "deep roots in shallow soil → full access");
    }

    #[test]
    fn root_zone_fraction_shallow_roots_deep_soil() {
        let grass = RootSystem::grass(); // 0.3m roots
        let frac = grass.root_zone_fraction(1.0); // 1m soil
        assert!((frac - 0.3).abs() < 0.01, "got {frac}");
    }

    #[test]
    fn root_zone_fraction_zero_depth() {
        assert_eq!(RootSystem::oak().root_zone_fraction(0.0), 0.0);
    }

    // --- water_uptake_mm tests ---

    #[test]
    fn uptake_limited_by_demand() {
        let oak = RootSystem::oak(); // 200 L/day capacity
        let soil = SoilWater::loam(); // at field capacity, plenty of water
        let uptake = oak.water_uptake_mm(&soil, 3.0); // only need 3mm
        assert!((uptake - 3.0).abs() < 0.01, "got {uptake}");
    }

    #[test]
    fn uptake_limited_by_capacity() {
        let grass = RootSystem::grass(); // 0.5 L/day capacity
        let soil = SoilWater::loam();
        let uptake = grass.water_uptake_mm(&soil, 100.0); // huge demand
        assert!((uptake - 0.5).abs() < 0.01, "got {uptake}");
    }

    #[test]
    fn uptake_limited_by_available_water() {
        let oak = RootSystem::oak();
        let mut soil = SoilWater::loam();
        soil.water_content_mm = soil.wilting_point_mm + 1.0; // barely any available
        let uptake = oak.water_uptake_mm(&soil, 100.0);
        assert!(
            uptake <= 1.0,
            "can't extract more than available, got {uptake}"
        );
    }

    #[test]
    fn uptake_zero_demand() {
        let oak = RootSystem::oak();
        let soil = SoilWater::loam();
        assert_eq!(oak.water_uptake_mm(&soil, 0.0), 0.0);
    }

    #[test]
    fn uptake_at_wilting_point() {
        let oak = RootSystem::oak();
        let mut soil = SoilWater::loam();
        soil.water_content_mm = soil.wilting_point_mm;
        assert_eq!(oak.water_uptake_mm(&soil, 5.0), 0.0);
    }

    #[test]
    fn shallow_roots_less_uptake_in_deep_soil() {
        use crate::water::SoilType;
        let mut soil = SoilWater::new(SoilType::Loam, 2.0); // 2m deep
        // Set to half available water
        soil.water_content_mm = soil.wilting_point_mm + soil.available_capacity_mm() * 0.5;
        let deep = RootSystem::oak(); // 5m roots → full access
        let shallow = RootSystem::grass(); // 0.3m roots → 15% access
        let demand = 3.0;
        let deep_up = deep.water_uptake_mm(&soil, demand);
        let shallow_up = shallow.water_uptake_mm(&soil, demand);
        // Shallow roots limited by zone available water (and capacity 0.5)
        assert!(shallow_up < deep_up, "shallow={shallow_up}, deep={deep_up}");
    }
}
