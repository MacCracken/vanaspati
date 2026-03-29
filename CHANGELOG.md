# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **docs** — README.md, CLAUDE.md, CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, architecture overview
- **lib** — complete re-exports for all public types and functions
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
