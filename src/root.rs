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
}
