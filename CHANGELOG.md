# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **water** — `SoilType` enum (Sand, SandyLoam, Loam, ClayLoam, Clay), `SoilWater` struct with hydraulic properties (Saxton & Rawls 2006), presets (sand, loam, clay), `add_water`/`remove_water`/`drain` state mutation, `relative_water_content`, `available_water_mm`, `deficit_mm`, `saturated_conductivity` (Rawls 1982), `infiltration_rate` (Green-Ampt), `soil_evaporation` (Philip stage-2), `WaterFluxes` summary struct, `daily_water_balance` combining rainfall→infiltration→drainage→transpiration→evaporation
- **root** — `root_zone_fraction(soil_depth)` scales water access by root depth vs soil depth; `water_uptake_mm(soil, demand)` extracts water limited by root capacity, zone available water, and transpiration demand
- **photosynthesis** — `water_stress_factor(rwc)` direct drought stress on photosynthesis (Sinclair & Ludlow 1986 threshold at RWC=0.4)
- **growth** — `water_stress_growth_factor(rwc)` drought stress on growth (Hsiao 1973, threshold at RWC=0.6 — growth declines before photosynthesis); `growth_stage(height, max_height)` maps height fraction to lifecycle stage
- **bridge** — `soil_water_to_photosynthesis_stress(rwc)` and `soil_water_to_growth_stress(rwc)` cross-crate water stress bridges; pravash bridges: `wind_to_boundary_conductance`, `humidity_to_vpd`
- **stomata** — Ball-Berry stomatal conductance model (`ball_berry_conductance`), `saturation_vapor_pressure` (Magnus-Tetens), `vapor_pressure_deficit`, `transpiration_rate`, `instantaneous_wue`, `drought_stomatal_factor` (anisohydric/isohydric), `vpd_stomatal_factor`, `boundary_layer_conductance` (Jones 2014), `total_leaf_conductance` (series resistance), `StomatalBehavior` enum
- **phenology** — `PhenologicalEvent` enum (DormancyBreak, BudBreak, LeafOut, Flowering, FruitSet, LeafSenescence, DormancyOnset), `growing_degree_days` and `accumulated_gdd` heat sum accumulation, `gdd_threshold` with literature values for temperate deciduous trees, `event_reached` and `phenological_progress`, `chilling_contribution` and `accumulated_chill` (Utah model, 0–7.2°C), `dormancy_broken`, `senescence_triggered` (photoperiod + temperature), `dormancy_onset_triggered` (short days or frost), `event_to_growth_stage` mapping

### Fixed
- **photosynthesis** — negative `quantum_yield` now returns 0 instead of negative rate
- **photosynthesis** — negative `extinction_k` in canopy functions now guarded (was exponential overflow)
- **season** — `daylight_hours_at` at ±90° latitude no longer produces wrong results (tan singularity clamped)
- **root** — added `RootSystem::mangrove()` preset for adventitious root type
- **Cargo.toml** — removed unused `hisab` dependency (81 → 79 deps)

## [0.2.0] - 2026-03-29

### Added
- **dispersal** — `DispersalMethod` enum (Wind, Animal, Gravity, Water, Explosive), `SeedProfile` struct with presets (dandelion, maple, acorn, coconut), `dispersal_distance` with method-specific formulas, `dispersal_probability` with exponential decay kernel
- **biomass** — `BiomassPool` struct with presets (oak, bamboo, grass), `AllocationStrategy` enum (Balanced, StressedRoot, Reproductive), `allocate` carbon partitioning, `height_to_diameter` and `height_to_leaf_area` allometric scaling
- **mortality** — `MortalityCause` enum, `age_mortality_rate` (Weibull hazard), `self_thinning_mortality` (Yoda's -3/2 power law), `frost_mortality` (logistic threshold), `drought_mortality` (quadratic deficit)
- **photosynthesis** — `PhotosynthesisPathway` enum (C3, C4, CAM), `pathway_params`, `temperature_factor_c4` (σ²=150, optimum 32°C), `temperature_factor_cam` (σ²=250, optimum 28°C); Beer-Lambert canopy light: `canopy_light_at_depth`, `understory_light_fraction`, `light_interception`, `shaded_photosynthesis_rate`
- **season** — `daylight_hours_at(day, latitude)` using sunrise equation, `growth_modifier_at(day, latitude)` continuous daylight-based modifier, `Season::from_day_latitude` hemisphere-aware
- **decomposition** — `LitterType` enum (Leaf, FineRoot, Wood, Reproductive), Q10 temperature factor, moisture bell curve, exponential decay (`remaining_mass`, `mass_decomposed`), `nitrogen_release` with C:N ratio, `half_life_days`
- **bridge** — 13 cross-crate bridge functions: badal (solar_to_par, atmosphere_to_photosynthesis_inputs, rainfall_to_water_supply, frost_risk_to_mortality, frost_to_dormancy, wind_to_dispersal_speed, growing_conditions_to_growth_multiplier), ushma (soil_temperature_to_root_activity, soil_temperature_to_growth_factor, evapotranspiration_cooling, wet_bulb_to_heat_stress), jantu (canopy_to_habitat_score, seed_production_to_food)
- **integration/soorat** — feature-gated `soorat-compat` visualization: `GrowthVisualization`, `RootVisualization`, `EcosystemMap`, `SeasonalColor`

## [0.1.0] - 2026-03-29

### Added
- **growth** — `GrowthStage` enum, `GrowthModel` struct with logistic growth, presets (oak, bamboo, grass)
- **photosynthesis** — light response curve, light compensation point, water use efficiency, temperature factor
- **season** — `Season` enum, day-of-year mapping (NH), daylight hours, growth modifiers
- **root** — `RootType` enum, `RootSystem` struct with depth/spread/uptake, presets (oak, grass), stabilization factor
- **pollination** — `PollinationMethod` enum, distance-based probability
- **ecosystem** — Lotka-Volterra competition, Shannon-Wiener diversity, net primary productivity
- **error** — `VanaspatiError` enum, `Result` type alias
- **logging** — optional tracing subscriber via `logging` feature
- **docs** — README.md, CLAUDE.md, CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, architecture overview
- **lib** — complete re-exports for all public types and functions
- **all modules** — tracing instrumentation, unit documentation with physical units

### Changed
- **season** — `from_day` clamps input to 1–365 instead of silently mapping invalid days
- **logging** — `init()` uses `try_init()` to avoid panic on double initialization
- **pollination** — guard against negative distances
- **Cargo.toml** — deprecated SPDX identifier `GPL-3.0` → `GPL-3.0-only`
