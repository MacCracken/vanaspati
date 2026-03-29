# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **bridge** — 13 cross-crate bridge functions: badal (weather→growth: solar_to_par, atmosphere_to_photosynthesis_inputs, rainfall_to_water_supply, frost_risk_to_mortality, frost_to_dormancy, wind_to_dispersal_speed, growing_conditions_to_growth_multiplier), ushma (thermo→physiology: soil_temperature_to_root_activity, soil_temperature_to_growth_factor, evapotranspiration_cooling, wet_bulb_to_heat_stress), jantu (plant→creature: canopy_to_habitat_score, seed_production_to_food)
- **dispersal** — `DispersalMethod` enum (Wind, Animal, Gravity, Water, Explosive), `SeedProfile` struct with presets (dandelion, maple, acorn, coconut), `dispersal_distance` with method-specific formulas, `dispersal_probability` with exponential decay kernel
- **biomass** — `BiomassPool` struct with presets (oak, bamboo, grass), `AllocationStrategy` enum (Balanced, StressedRoot, Reproductive), `allocate` carbon partitioning, `height_to_diameter` and `height_to_leaf_area` allometric scaling functions
- **mortality** — `MortalityCause` enum, `age_mortality_rate` (Weibull hazard), `self_thinning_mortality` (Yoda's -3/2 power law), `frost_mortality` (logistic threshold), `drought_mortality` (quadratic deficit)
- **photosynthesis** — `PhotosynthesisPathway` enum (C3, C4, CAM), `pathway_params` returning (optimum_temp, quantum_yield, max_rate), `temperature_factor_c4` (σ²=150, optimum 32°C), `temperature_factor_cam` (σ²=250, optimum 28°C)
- **season** — `daylight_hours_at(day, latitude)` using sunrise equation with solar declination, `growth_modifier_at(day, latitude)` continuous daylight-based modifier, `Season::from_day_latitude` hemisphere-aware season mapping
- **docs** — README.md, CLAUDE.md, CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, architecture overview
- **lib** — complete re-exports for all public types and functions (9 modules, 30+ items)
- **all modules** — tracing instrumentation on all operations
- **all modules** — unit documentation with parameter descriptions and physical units
- **examples** — real `basic.rs` example demonstrating all modules

### Changed
- **season** — `from_day` now clamps input to 1–365 instead of silently mapping invalid days
- **logging** — `init()` uses `try_init()` to avoid panic on double initialization
- **pollination** — guard against negative distances

### Fixed
- **season** — `winter_solstice_is_winter` test was asserting `Autumn` for day 355 (boundary, not a code bug — test adjusted to day 356)
- **Cargo.toml** — deprecated SPDX identifier `GPL-3.0` → `GPL-3.0-only`

## [0.1.0] - 2025-01-01

### Added
- **growth** — `GrowthStage` enum, `GrowthModel` struct with logistic growth, presets (oak, bamboo, grass)
- **photosynthesis** — light response curve, light compensation point, water use efficiency, temperature factor
- **season** — `Season` enum, day-of-year mapping (NH), daylight hours, growth modifiers
- **root** — `RootType` enum, `RootSystem` struct with depth/spread/uptake, presets (oak, grass), stabilization factor
- **pollination** — `PollinationMethod` enum, distance-based probability
- **ecosystem** — Lotka-Volterra competition, Shannon-Wiener diversity, net primary productivity
- **error** — `VanaspatiError` enum, `Result` type alias
- **logging** — optional tracing subscriber via `logging` feature
