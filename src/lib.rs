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
//! | [`decomposition`] | Litter decay, temperature/moisture factors, nutrient release |
//! | [`water`] | Soil water storage, infiltration, drainage, evaporation |
//! | [`stomata`] | Ball-Berry stomatal conductance, transpiration, VPD, boundary layer |
//! | [`phenology`] | Growing degree days, chilling hours, lifecycle event triggers |
//! | [`nitrogen`] | Soil N pools, mineralization, uptake, leaching, N stress |
//! | [`herbivory`] | Grazing/browsing biomass removal, compensatory growth |
//! | [`succession`] | Pioneer/climax dynamics, shade tolerance, establishment |
//! | [`reproduction`] | Vegetative reproduction — runners, rhizomes, clonal spread |
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
pub mod decomposition;
pub mod dispersal;
pub mod ecosystem;
pub mod error;
pub mod growth;
pub mod herbivory;
/// Integration APIs for downstream consumers (soorat rendering).
pub mod integration;
pub mod mortality;
pub mod nitrogen;
pub mod phenology;
pub mod photosynthesis;
pub mod pollination;
pub mod reproduction;
pub mod root;
pub mod season;
pub mod stomata;
pub mod succession;
pub mod water;

#[cfg(feature = "logging")]
pub mod logging;

// Error
pub use error::{Result, VanaspatiError};

// Growth
pub use growth::{GrowthModel, GrowthStage, growth_stage, water_stress_growth_factor};

// Photosynthesis
pub use photosynthesis::{
    PhotosynthesisPathway, canopy_light_at_depth, light_compensation_point, light_interception,
    pathway_params, photosynthesis_rate, shaded_photosynthesis_rate, temperature_factor,
    temperature_factor_c4, temperature_factor_cam, understory_light_fraction, water_stress_factor,
    water_use_efficiency,
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

// Water
pub use water::{
    SoilType, SoilWater, WaterFluxes, daily_water_balance, infiltration_rate,
    saturated_conductivity, soil_evaporation,
};

// Stomata
pub use stomata::{
    StomatalBehavior, ball_berry_conductance, boundary_layer_conductance, drought_stomatal_factor,
    instantaneous_wue, saturation_vapor_pressure, total_leaf_conductance, transpiration_rate,
    vapor_pressure_deficit, vpd_stomatal_factor,
};

// Phenology
pub use phenology::{
    PhenologicalEvent, accumulated_chill, accumulated_gdd, chilling_contribution, dormancy_broken,
    dormancy_onset_triggered, event_reached, event_to_growth_stage, gdd_threshold,
    growing_degree_days, phenological_progress, senescence_triggered,
};

// Decomposition
pub use decomposition::{
    LitterType, base_decomposition_rate, daily_decomposition_rate, half_life_days, mass_decomposed,
    moisture_decomposition_factor, nitrogen_release, remaining_mass,
    temperature_decomposition_factor,
};

// Nitrogen
pub use nitrogen::{
    NitrogenFluxes, SoilNitrogen, critical_n_concentration, daily_nitrogen_balance,
    mineralization_rate, nitrogen_leaching, nitrogen_stress_factor, nitrogen_uptake,
    plant_n_demand,
};

// Herbivory
pub use herbivory::{
    HerbivoryType, biomass_removal, compensatory_growth_factor, herbivory_mortality,
    organ_vulnerability, total_biomass_removed,
};

// Succession
pub use succession::{
    SuccessionalStage, competitive_displacement, effective_growth_multiplier,
    establishment_probability, max_growth_rate_multiplier, shade_tolerance, typical_lifespan_years,
};

// Reproduction (vegetative)
pub use reproduction::{
    VegetativeMethod, base_ramet_rate, clonal_area_m2, parent_cost_kg, ramet_cost_fraction,
    resource_limited_ramets, spread_distance_m,
};

// Ecosystem
pub use ecosystem::{competition_growth, net_primary_productivity, shannon_diversity};

// Bridge
pub use bridge::{
    atmosphere_to_photosynthesis_inputs, canopy_to_habitat_score, evapotranspiration_cooling,
    frost_risk_to_mortality, frost_to_dormancy, growing_conditions_to_growth_multiplier,
    herbivore_to_biomass_loss, humidity_to_vpd, light_to_successional_advantage,
    nitrogen_to_growth_stress, rainfall_to_water_supply, seed_production_to_food,
    soil_temperature_to_growth_factor, soil_temperature_to_root_activity,
    soil_water_to_growth_stress, soil_water_to_photosynthesis_stress, solar_to_par,
    wet_bulb_to_heat_stress, wind_to_boundary_conductance, wind_to_dispersal_speed,
};
