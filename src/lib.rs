//! # Vanaspati
//!
//! **Vanaspati** (वनस्पति — Sanskrit for "lord of the forest, plant/tree") — botany
//! and plant behavior engine for the AGNOS ecosystem.
//!
//! Provides plant growth models, photosynthesis, seasonal cycles, root systems,
//! pollination, seed dispersal, biomass allocation, mortality, and ecosystem dynamics.
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`growth`] | Logistic growth model with stage progression and species presets |
//! | [`photosynthesis`] | Light response curves, C3/C4/CAM pathways, temperature factors |
//! | [`season`] | Day-of-year mapping, latitude-aware daylight hours, growth modifiers |
//! | [`root`] | Root system types, depth/spread, water uptake, soil stabilization |
//! | [`pollination`] | Pollination methods and distance-based probability |
//! | [`dispersal`] | Seed dispersal methods, distance kernels, seed profiles |
//! | [`biomass`] | Carbon allocation across plant organs, allometric scaling |
//! | [`mortality`] | Age, drought, frost, competition mortality models |
//! | [`ecosystem`] | Lotka-Volterra competition, Shannon diversity, NPP |
//! | [`bridge`] | Cross-crate conversions for badal, ushma, jantu |
//! | [`error`] | Error types |
//!
//! ## Quick Start
//!
//! ```
//! use vanaspati::{GrowthModel, Season, photosynthesis_rate, growth_stage, daylight_hours_at};
//!
//! // Grow an oak for a year
//! let oak = GrowthModel::oak();
//! let height = oak.height_at_day(365.0);
//! let stage = growth_stage(height, oak.max_height);
//!
//! // Photosynthesis under full sun
//! let rate = photosynthesis_rate(20.0, 0.05, 800.0);
//!
//! // Latitude-aware daylight
//! let hours = daylight_hours_at(172, 45.0); // summer solstice, 45°N
//! assert!(hours > 15.0);
//! ```

pub mod biomass;
pub mod bridge;
pub mod dispersal;
pub mod ecosystem;
pub mod error;
pub mod growth;
pub mod mortality;
pub mod photosynthesis;
pub mod pollination;
pub mod root;
pub mod season;

#[cfg(feature = "logging")]
pub mod logging;

// Error
pub use error::{Result, VanaspatiError};

// Growth
pub use growth::{GrowthModel, GrowthStage, growth_stage};

// Photosynthesis
pub use photosynthesis::{
    PhotosynthesisPathway, light_compensation_point, pathway_params, photosynthesis_rate,
    temperature_factor, temperature_factor_c4, temperature_factor_cam, water_use_efficiency,
};

// Season
pub use season::{Season, daylight_hours_at, growth_modifier_at};

// Root
pub use root::{RootSystem, RootType};

// Pollination
pub use pollination::{PollinationMethod, pollination_probability};

// Dispersal
pub use dispersal::{DispersalMethod, SeedProfile, dispersal_distance, dispersal_probability};

// Biomass
pub use biomass::{
    AllocationStrategy, BiomassPool, allocate, height_to_diameter, height_to_leaf_area,
};

// Mortality
pub use mortality::{
    MortalityCause, age_mortality_rate, drought_mortality, frost_mortality, self_thinning_mortality,
};

// Ecosystem
pub use ecosystem::{competition_growth, net_primary_productivity, shannon_diversity};

// Bridge
pub use bridge::{
    atmosphere_to_photosynthesis_inputs, canopy_to_habitat_score, evapotranspiration_cooling,
    frost_risk_to_mortality, frost_to_dormancy, growing_conditions_to_growth_multiplier,
    rainfall_to_water_supply, seed_production_to_food, soil_temperature_to_growth_factor,
    soil_temperature_to_root_activity, solar_to_par, wet_bulb_to_heat_stress,
    wind_to_dispersal_speed,
};
