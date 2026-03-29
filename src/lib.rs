//! # Vanaspati
//!
//! **Vanaspati** (वनस्पति — Sanskrit for "lord of the forest, plant/tree") — botany
//! and plant behavior engine for the AGNOS ecosystem.
//!
//! Provides plant growth models, photosynthesis, seasonal cycles, root systems,
//! pollination, seed dispersal, and ecosystem dynamics.

pub mod error;
pub mod growth;
pub mod photosynthesis;
pub mod season;
pub mod root;
pub mod pollination;
pub mod ecosystem;

#[cfg(feature = "logging")]
pub mod logging;

pub use error::{VanaspatiError, Result};
pub use growth::{GrowthModel, GrowthStage};
pub use photosynthesis::photosynthesis_rate;
pub use season::Season;
