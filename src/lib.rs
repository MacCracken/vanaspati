//! # Vanaspati
//!
//! **Vanaspati** (वनस्पति — Sanskrit for "lord of the forest, plant/tree") — botany
//! and plant behavior engine for the AGNOS ecosystem.
//!
//! Provides plant growth models, photosynthesis, seasonal cycles, root systems,
//! pollination, and ecosystem dynamics.
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`growth`] | Logistic growth model with stage progression and species presets |
//! | [`photosynthesis`] | Light response curves, compensation point, WUE, temperature factor |
//! | [`season`] | Day-of-year to season mapping, daylight hours, growth modifiers |
//! | [`root`] | Root system types, depth/spread, water uptake, soil stabilization |
//! | [`pollination`] | Pollination methods and distance-based probability |
//! | [`ecosystem`] | Lotka-Volterra competition, Shannon diversity, NPP |
//! | [`error`] | Error types |

pub mod ecosystem;
pub mod error;
pub mod growth;
pub mod photosynthesis;
pub mod pollination;
pub mod root;
pub mod season;

#[cfg(feature = "logging")]
pub mod logging;

// Error
pub use error::{Result, VanaspatiError};

// Growth
pub use growth::{GrowthModel, GrowthStage};

// Photosynthesis
pub use photosynthesis::{
    light_compensation_point, photosynthesis_rate, temperature_factor, water_use_efficiency,
};

// Season
pub use season::Season;

// Root
pub use root::{RootSystem, RootType};

// Pollination
pub use pollination::{PollinationMethod, pollination_probability};

// Ecosystem
pub use ecosystem::{competition_growth, net_primary_productivity, shannon_diversity};
