//! Soorat integration — visualization data structures for botanical simulation.
//!
//! Provides structured types that soorat can render: growth stage visuals,
//! root systems, ecosystem density maps, and seasonal color palettes.

use serde::{Deserialize, Serialize};

// ── Growth stage visualization ─────────────────────────────────────────────

/// Plant growth parameters for procedural mesh/billboard rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrowthVisualization {
    /// Current height in metres.
    pub height: f32,
    /// Maximum possible height in metres.
    pub max_height: f32,
    /// Growth stage name (e.g. "Seedling", "Vegetative", "Flowering").
    pub stage: String,
    /// Trunk/stem diameter estimate in metres (allometric from height).
    pub diameter: f32,
    /// Leaf area index (m² leaf / m² ground).
    pub leaf_area: f32,
    /// Growth fraction (0.0–1.0): current height / max height.
    pub maturity: f32,
}

impl GrowthVisualization {
    /// Create from a growth model at a given day.
    #[must_use]
    pub fn from_model(model: &crate::growth::GrowthModel, day: f32) -> Self {
        let height = model.height_at_day(day);
        let stage = crate::growth::growth_stage(height, model.max_height);
        // Default species coefficient for allometry
        let diameter = crate::biomass::height_to_diameter(height, 0.05);
        let leaf_area = crate::biomass::height_to_leaf_area(height, 0.3);
        let maturity = if model.max_height > 0.0 {
            (height / model.max_height).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            height,
            max_height: model.max_height,
            stage: format!("{stage:?}"),
            diameter,
            leaf_area,
            maturity,
        }
    }
}

// ── Root system visualization ──────────────────────────────────────────────

/// Root system data for line/volume rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootVisualization {
    /// Root type name (e.g. "Taproot", "Fibrous").
    pub root_type: String,
    /// Maximum depth in metres.
    pub max_depth: f32,
    /// Spread radius in metres.
    pub spread_radius: f32,
    /// Water uptake rate (m³/s).
    pub water_uptake_rate: f32,
    /// Stabilization factor (0.0–1.0).
    pub stabilization: f32,
}

impl RootVisualization {
    /// Create from a vanaspati `RootSystem`.
    #[must_use]
    pub fn from_root_system(root: &crate::root::RootSystem) -> Self {
        Self {
            root_type: format!("{:?}", root.root_type),
            max_depth: root.max_depth_m,
            spread_radius: root.spread_radius_m,
            water_uptake_rate: root.water_uptake_rate,
            stabilization: root.stabilization_factor(),
        }
    }
}

// ── Ecosystem map ──────────────────────────────────────────────────────────

/// Species density grid for heatmap rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcosystemMap {
    /// Population density at each cell. Flattened row-major: `density[y * nx + x]`.
    pub density: Vec<f64>,
    /// Grid dimensions (nx, ny).
    pub dimensions: [usize; 2],
    /// Cell size in metres.
    pub cell_size: f64,
    /// Species name.
    pub species: String,
    /// Maximum density for normalization.
    pub max_density: f64,
}

impl EcosystemMap {
    /// Create a uniform density map.
    #[must_use]
    pub fn uniform(
        nx: usize,
        ny: usize,
        cell_size: f64,
        species: &str,
        density_value: f64,
    ) -> Self {
        Self {
            density: vec![density_value; nx * ny],
            dimensions: [nx, ny],
            cell_size,
            species: species.to_string(),
            max_density: density_value,
        }
    }
}

// ── Seasonal color palette ─────────────────────────────────────────────────

/// Seasonal foliage color for material tinting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeasonalColor {
    /// Season name.
    pub season: crate::season::Season,
    /// Foliage RGB color (0.0–1.0 each).
    pub foliage_rgb: [f32; 3],
    /// Growth modifier (0.0–1.0) for the season.
    pub growth_modifier: f32,
    /// Daylight hours at this latitude/day.
    pub daylight_hours: f32,
}

impl SeasonalColor {
    /// Compute seasonal color for a given day of year and latitude.
    #[must_use]
    pub fn at_day(day_of_year: u16, latitude_deg: f32) -> Self {
        let season = crate::season::Season::from_day(day_of_year);
        let growth = crate::season::growth_modifier_at(day_of_year, latitude_deg);
        let daylight = crate::season::daylight_hours_at(day_of_year, latitude_deg);

        let foliage_rgb = match season {
            crate::season::Season::Spring => [0.4, 0.8, 0.3], // light green
            crate::season::Season::Summer => [0.2, 0.6, 0.15], // deep green
            crate::season::Season::Autumn => [0.8, 0.5, 0.15], // orange/brown
            crate::season::Season::Winter => [0.5, 0.45, 0.35], // bare/grey-brown
        };

        Self {
            season,
            foliage_rgb,
            growth_modifier: growth,
            daylight_hours: daylight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_viz_oak() {
        let model = crate::growth::GrowthModel::oak();
        let viz = GrowthVisualization::from_model(&model, 365.0 * 5.0);
        assert!(viz.height > 0.0);
        assert!(viz.diameter > 0.0);
        assert!(viz.maturity > 0.0 && viz.maturity <= 1.0);
    }

    #[test]
    fn growth_viz_day_zero() {
        let model = crate::growth::GrowthModel::oak();
        let viz = GrowthVisualization::from_model(&model, 0.0);
        assert!(viz.height > 0.0); // initial height
        assert!(viz.maturity < 0.1);
    }

    #[test]
    fn root_viz_oak() {
        let root = crate::root::RootSystem::oak();
        let viz = RootVisualization::from_root_system(&root);
        assert!(viz.max_depth > 1.0);
        assert!(viz.spread_radius > 0.0);
        assert!(viz.stabilization > 0.0);
        assert!(viz.root_type.contains("Taproot"));
    }

    #[test]
    fn root_viz_grass() {
        let root = crate::root::RootSystem::grass();
        let viz = RootVisualization::from_root_system(&root);
        assert!(viz.root_type.contains("Fibrous"));
    }

    #[test]
    fn ecosystem_map_uniform() {
        let map = EcosystemMap::uniform(5, 5, 10.0, "Oak", 100.0);
        assert_eq!(map.density.len(), 25);
        assert!((map.max_density - 100.0).abs() < 0.01);
    }

    #[test]
    fn seasonal_color_summer() {
        let sc = SeasonalColor::at_day(180, 45.0);
        assert!(matches!(sc.season, crate::season::Season::Summer));
        assert!(sc.growth_modifier > 0.5);
        assert!(sc.daylight_hours > 14.0);
        assert!(sc.foliage_rgb[1] > sc.foliage_rgb[0]);
    }

    #[test]
    fn seasonal_color_winter() {
        let sc = SeasonalColor::at_day(15, 45.0);
        assert!(matches!(sc.season, crate::season::Season::Winter));
        assert!(sc.growth_modifier < 0.3);
    }

    #[test]
    fn seasonal_color_serializes() {
        let sc = SeasonalColor::at_day(100, 30.0);
        let json = serde_json::to_string(&sc);
        assert!(json.is_ok());
    }
}
