# Vanaspati Roadmap

> **Vanaspati** (Sanskrit: lord of the forest, plant/tree) — botany and plant behavior engine for growth, photosynthesis, seasonal cycles, and ecosystems.

## Scope

Vanaspati owns the **science of plants**: growth models, photosynthesis, seasonal cycles, root systems, pollination, seed dispersal, and ecosystem dynamics. It provides the math; consumers decide what to do with it (render forests, simulate ecosystems, model agriculture).

Vanaspati does NOT own:
- **Math primitives** -> hisab (vectors, geometry, calculus)
- **Rendering** -> soorat/kiran (they consume vanaspati for vegetation visualization)
- **Animal behavior** -> jantu (animals interact with plants via bridges)
- **Weather/climate** -> badal (weather drives plant growth via bridges)
- **Sound synthesis** -> garjan (foliage rustling via garjan's bridge)

---

## Backlog

> Not scheduled — demand-gated.

### High Priority

- [x] **Water/soil moisture system** — precipitation, soil water storage, root uptake, drought stress on growth/photosynthesis. Single biggest missing piece — water is the primary limiting factor for plant growth in most terrestrial ecosystems
- [x] **Nutrient system (nitrogen)** — soil N pool, uptake, effect on growth rate. Nitrogen is the most commonly limiting nutrient
### Medium Priority

- [x] Mycorrhizal network (plant-fungal nutrient exchange)
- [x] Allelopathy (chemical competition between plants)
- [x] Fire ecology (fire-adapted species, post-fire regeneration)
- [x] **Herbivory pressure** — grazing/browsing biomass removal
- [x] **Vegetative reproduction** — runners, rhizomes, root sprouting (clonal spread)
- [x] **Succession dynamics** — pioneer vs. climax species, shade tolerance, lifespan classes
- [x] Remaining mortality types — fire, disease, windthrow

### Post-v1.0

- [ ] **Farquhar (FvCB) photosynthesis** — Rubisco/RuBP/TPU-limited biochemical model (feature-gated alongside simplified light response)
- [ ] Multi-layer soil hydrology (2–3 layers)
- [ ] Phosphorus cycle
- [ ] Snow accumulation/melt
- [ ] Soil temperature model
- [ ] Litter quality chemistry (lignin:N ratio)
- [ ] Cohort/individual tracking (size/age-structured)
- [ ] Management operations (tillage, irrigation, harvest, thinning)
- [ ] Spatial grid / landscape processes

---

## Completed

### V0.1.0 — Foundation

- **growth** — GrowthStage enum, GrowthModel (logistic curve), presets (oak, bamboo, grass), `height_at_day`, `daily_growth`, `growth_stage`
- **photosynthesis** — light response curve, light compensation point, water use efficiency, temperature factor (C3)
- **season** — Season enum, day-of-year mapping (NH), daylight hours, growth modifiers
- **root** — RootType enum, RootSystem with depth/spread/uptake, presets (oak, grass, mangrove), stabilization factor
- **pollination** — PollinationMethod enum, distance-based probability
- **ecosystem** — Lotka-Volterra competition, Shannon-Wiener diversity, net primary productivity
- **error** — VanaspatiError enum, Result type
- **logging** — optional tracing subscriber via `logging` feature

### V0.1.1 — Scaffold Hardening (P-1)

- Full `cargo fmt` cleanup, tracing on all operations, complete lib.rs re-exports
- CHANGELOG, CONTRIBUTING, SECURITY, architecture overview documentation
- Fixed: deprecated SPDX identifier, season test boundary, logging panic on double-init
- Tests: 28 → 61, real benchmarks replacing placeholders

### V0.2.0 — Modules & Bridges

- **dispersal** — DispersalMethod enum (Wind, Animal, Gravity, Water, Explosive), SeedProfile presets, exponential decay kernel
- **biomass** — BiomassPool with presets, AllocationStrategy, allometric scaling (height→diameter, height→leaf area)
- **mortality** — MortalityCause enum, age (Weibull), drought (quadratic), frost (logistic), self-thinning (Yoda -3/2)
- **photosynthesis** — C4/CAM pathways (PhotosynthesisPathway enum, pathway_params, type-specific temperature factors); Beer-Lambert canopy light competition (canopy_light_at_depth, understory_light_fraction, light_interception, shaded_photosynthesis_rate)
- **season** — latitude-parameterized: daylight_hours_at (sunrise equation), growth_modifier_at, from_day_latitude (hemisphere-aware)
- **decomposition** — LitterType enum, Q10 temperature factor, moisture bell curve, exponential decay, nitrogen release with C:N ratio, half-life
- **phenology** — PhenologicalEvent enum, GDD accumulation, chilling hours (Utah model), event thresholds, dormancy break, senescence/dormancy triggers, event_to_growth_stage mapping
- **stomata** — Ball-Berry conductance, VPD, transpiration, drought/VPD factors, boundary layer, total leaf conductance
- **bridge** — 15 cross-crate functions: badal (solar→PAR, weather→growth, frost→dormancy, wind→dispersal), ushma (soil temp→root activity, ET cooling, wet bulb stress), pravash (wind→boundary conductance, humidity→VPD), jantu (canopy→habitat, seeds→food)
- **integration/soorat** — feature-gated visualization: GrowthVisualization, RootVisualization, EcosystemMap, SeasonalColor

---

## Consumers

| Consumer | What it uses |
|----------|-------------|
| **kiran** | Vegetation in game worlds (forests, grass, crops) |
| **joshua** | Ecosystem simulation, agricultural modeling |
| **garjan** | Foliage rustling, branch creaking (via garjan's bridge) |
| **jantu** | Habitat/food source for creature behavior |

## Boundary with Other Crates

| Feature | vanaspati | other |
|---------|-----------|-------|
| Plant growth math | Yes | -- |
| Weather/climate data | -- | badal |
| Animal behavior | -- | jantu |
| Fluid dynamics (sap flow) | -- | pravash (future) |
| Vector/matrix math | -- | hisab |
| Vegetation rendering | -- | soorat/kiran |
| Sound synthesis (rustling) | -- | garjan |
