# Vanaspati — Architecture Overview

## Module Map

```text
vanaspati/
├── Core plant physiology
│   ├── growth.rs          — logistic growth model, stage enum, presets, water stress factor
│   ├── photosynthesis.rs  — C3/C4/CAM pathways, light response, canopy competition, water stress
│   ├── stomata.rs         — Ball-Berry conductance, VPD, transpiration, drought/VPD factors
│   ├── root.rs            — root types, depth/spread, water uptake, zone fraction, stabilization
│   ├── biomass.rs         — organ pools, allocation strategies, allometric scaling
│   └── phenology.rs       — GDD, chilling hours, phenological events, dormancy
│
├── Soil & resources
│   ├── water.rs           — soil types, water storage, infiltration, drainage, daily balance
│   ├── nitrogen.rs        — N pools, mineralization, uptake, leaching, stress factor
│   └── decomposition.rs   — litter types, Q10 decay, nitrogen release
│
├── Reproduction & dispersal
│   ├── pollination.rs     — pollination methods, distance-based probability
│   ├── dispersal.rs       — dispersal methods, seed profiles, exponential kernel
│   └── reproduction.rs    — vegetative methods, clonal spread, ramet production
│
├── Disturbance & interactions
│   ├── mortality.rs       — age, drought, frost, fire, disease, windthrow, self-thinning
│   ├── herbivory.rs       — grazing/browsing, compensatory growth, herbivory mortality
│   ├── fire.rs            — fire strategies, bark protection, resprouting, serotiny
│   ├── allelopathy.rs     — allelochemical production, soil accumulation, dose-response
│   └── mycorrhiza.rs      — fungal types, nutrient enhancement, colonization, hyphal reach
│
├── Community dynamics
│   ├── succession.rs      — pioneer/climax, shade tolerance, establishment, displacement
│   ├── ecosystem.rs       — competition, diversity, NPP
│   └── season.rs          — season enum, daylight hours, growth modifiers
│
├── Cross-crate integration
│   ├── bridge.rs          — 20+ adapter functions for badal, ushma, pravash, jantu
│   └── integration/
│       └── soorat.rs      — visualization types (feature-gated: soorat-compat)
│
├── Infrastructure
│   ├── error.rs           — VanaspatiError enum
│   └── logging.rs         — optional tracing subscriber (feature-gated: logging)
│
└── lib.rs                 — public re-exports, module documentation
```

## Data Flow

Vanaspati is a **computation library** — it takes parameters in, returns numbers out. There is no internal state, no ECS, no runtime. Consumers call functions directly.

```text
Consumer (kiran, joshua, jantu)
    │
    ├─ Growth: GrowthModel::oak().daily_growth(height)
    ├─ Photosynthesis: photosynthesis_rate(pmax, alpha, par) × water_stress_factor(rwc)
    ├─ Water: daily_water_balance(&mut soil, rain, transpiration, evaporation)
    ├─ Nitrogen: daily_nitrogen_balance(&mut soil_n, temp, moisture, demand, roots, drain, water)
    ├─ Stomata: ball_berry_conductance(photo, co2, humidity) → transpiration_rate(gs, vpd)
    ├─ Phenology: accumulated_gdd(temps, base) → event_reached(gdd, event)
    ├─ Season: daylight_hours_at(day, latitude) → growth_modifier_at(day, latitude)
    ├─ Mortality: drought_mortality(avail, demand) + fire_mortality(intensity, bark)
    ├─ Ecosystem: competition_growth(N, r, K) + shannon_diversity(proportions)
    ├─ Herbivory: biomass_removal(leaf, stem, root, repro, type, intensity)
    ├─ Succession: effective_growth_multiplier(light, stage)
    ├─ Fire: bark_protection(strategy) → resprout_vigor(strategy, intensity)
    ├─ Mycorrhiza: enhanced_n_uptake(base, type, colonization)
    ├─ Allelopathy: soil_concentration(old, input, temp, moisture) → growth_inhibition(conc, sens)
    └─ Bridge: solar_to_par(), humidity_to_vpd(), herbivore_to_biomass_loss(), etc.
```

## Dependency Stack

```text
vanaspati
├── hisab            — math primitives (interpolation, transforms, numerics)
├── serde + derive   — serialization for save/load
├── thiserror        — error derive macros
├── tracing          — structured logging (always available)
└── tracing-subscriber (optional, feature = "logging")
```

## Consumers

| Crate     | What it uses                                                |
|-----------|-------------------------------------------------------------|
| kiran     | Growth, seasons, photosynthesis for vegetation rendering    |
| joshua    | Full ecosystem: water, nitrogen, succession, fire, herbivory|
| garjan    | Seasonal state for foliage sound synthesis                  |
| jantu     | Habitat/food via bridge, herbivory for feeding interactions |

## Design Principles

- **Pure functions** — no mutable global state, no side effects beyond tracing
- **f32 throughout** — game/simulation precision, consistent numeric type
- **Factory presets** — quick start with realistic defaults (oak, bamboo, grass, etc.)
- **`#[must_use]`** — all pure functions annotated to prevent silent result discard
- **`#[non_exhaustive]`** — all public enums are extensible without breaking changes
- **`#[inline]`** — hot-path and small functions hinted for inlining
- **Feature-gated optionals** — logging and soorat-compat are opt-in
- **Serializable** — all types derive `Serialize`/`Deserialize` for save/load
- **Structured logging** — every operation traced with parameters for audit trail
- **No `unwrap()`** — safe defaults and clamping throughout library code
