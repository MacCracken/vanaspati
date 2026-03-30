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

## V0.1.0 — Foundation (done)

### growth
- [x] GrowthStage enum (Seed, Germination, Seedling, Vegetative, Flowering, Fruiting, Senescence, Dormant)
- [x] GrowthModel struct (logistic growth curve)
- [x] Factory presets (oak, bamboo, grass)
- [x] `height_at_day()`, `daily_growth()` methods

### photosynthesis
- [x] Light response curve (`photosynthesis_rate`)
- [x] Light compensation point calculation
- [x] Water use efficiency
- [x] Temperature factor (bell curve, optimum ~25C for C3)

### season
- [x] Season enum (Spring, Summer, Autumn, Winter)
- [x] Day-of-year to season mapping (northern hemisphere)
- [x] Daylight hours by season
- [x] Growth modifier (0.0-1.0)

### root
- [x] RootType enum (Taproot, Fibrous, Adventitious)
- [x] RootSystem struct with depth, spread, water uptake
- [x] Factory presets (oak taproot, grass fibrous)
- [x] Stabilization factor

### pollination
- [x] PollinationMethod enum (Wind, Insect, Bird, Water, SelfPollinating)
- [x] Distance-based pollination probability

### ecosystem
- [x] Lotka-Volterra competition model
- [x] Shannon-Wiener diversity index
- [x] Net primary productivity

---

## Cross-Crate Bridges

- [x] **`bridge.rs` module** — primitive-value conversions for cross-crate botany (done: 13 functions)
- [x] **badal bridge**: solar_to_par, atmosphere_to_photosynthesis_inputs, rainfall_to_water_supply, frost_risk_to_mortality, frost_to_dormancy, wind_to_dispersal_speed, growing_conditions_to_growth_multiplier (done)
- [x] **jantu bridge**: canopy_to_habitat_score (Beer-Lambert LAI), seed_production_to_food (log saturation) (done)
- [x] **ushma bridge**: soil_temperature_to_root_activity, soil_temperature_to_growth_factor, evapotranspiration_cooling, wet_bulb_to_heat_stress (done)

---

## Soorat Integration (`integration/soorat.rs`)

> Feature-gated `soorat-compat` — structured visualization types for soorat rendering

- [ ] **`integration/soorat.rs` module** — visualization data structures
- [ ] **Growth stage visualization**: plant structure (trunk, branches, canopy) with growth parameters for procedural mesh generation
- [ ] **Root system**: branching structure with depth/spread for line rendering
- [ ] **Ecosystem map**: species distribution grid with density values for heatmap rendering
- [ ] **Seasonal color**: phenology stage -> foliage color palette for material tinting

---

## Future

> Not scheduled — demand-gated. Prioritized by domain research (P(-1) audit, 2026-03-29).

### High Priority (biggest realism gains)

- [ ] **Water/soil moisture system** — precipitation, soil water storage, root uptake, drought stress on growth/photosynthesis. Single biggest missing piece — water is the primary limiting factor for plant growth in most terrestrial ecosystems
- [ ] **Nutrient system (nitrogen)** — soil N pool, uptake, effect on growth rate. Nitrogen is the most commonly limiting nutrient
- [x] **Seed dispersal** — wind, animal, gravity, water, explosive dispersal with exponential decay kernels (done: V0.2.0)
- [x] **Mortality and disturbance** — age (Weibull), drought (quadratic deficit), frost (logistic), self-thinning (Yoda's -3/2) (done: V0.2.0). Remaining: fire, disease, windthrow
- [ ] **Phenology (growing degree days)** — replace/augment rigid season boundaries with accumulated heat + photoperiod triggers + chilling requirements

### Medium Priority

- [x] **Biomass allocation** — carbon partitioning (Balanced, StressedRoot, Reproductive), allometric scaling (height→diameter, height→leaf area) (done: V0.2.0)
- [x] **Light competition / canopy structure** — Beer-Lambert: canopy_light_at_depth, understory_light_fraction, light_interception, shaded_photosynthesis_rate (done)
- [x] C4/CAM photosynthesis pathways — PhotosynthesisPathway enum, pathway_params, type-specific temperature factors (done: V0.2.0)
- [x] Southern hemisphere / latitude-parameterized seasons — sunrise equation, daylight_hours_at, growth_modifier_at, from_day_latitude (done: V0.2.0)
- [x] **Decomposition and litter** — LitterType enum, Q10 temperature factor, moisture bell curve, exponential decay, nitrogen release with C:N ratio, half-life (done)
- [ ] **Stomatal conductance** — Ball-Berry model coupling photosynthesis, water loss, and temperature

### Lower Priority

- [ ] Mycorrhizal network (plant-fungal nutrient exchange)
- [ ] Allelopathy (chemical competition between plants)
- [ ] Fire ecology (fire-adapted species, post-fire regeneration)
- [ ] **Herbivory pressure** — grazing/browsing biomass removal
- [ ] **Vegetative reproduction** — runners, rhizomes, root sprouting (clonal spread)
- [ ] **Succession dynamics** — pioneer vs. climax species, shade tolerance, lifespan classes

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
