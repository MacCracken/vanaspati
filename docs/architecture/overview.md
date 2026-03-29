# Vanaspati — Architecture Overview

## Module Map

```text
vanaspati/
├── growth.rs          — logistic growth model, stage enum, species presets
├── photosynthesis.rs  — light response, compensation point, WUE, temperature
├── season.rs          — season enum, daylight hours, growth modifiers
├── root.rs            — root types, depth/spread, water uptake, stabilization
├── pollination.rs     — pollination methods, distance-based probability
├── ecosystem.rs       — competition, diversity, NPP
├── error.rs           — VanaspatiError enum
├── logging.rs         — optional tracing subscriber (feature-gated)
└── lib.rs             — public re-exports
```

## Data Flow

Vanaspati is a **computation library** — it takes parameters in, returns numbers out. There is no internal state, no ECS, no runtime. Consumers call functions directly.

```text
Consumer (kiran, joshua, jantu)
    │
    ├─ GrowthModel::oak().height_at_day(day)
    ├─ photosynthesis_rate(pmax, alpha, par)
    ├─ Season::from_day(day).growth_modifier()
    ├─ pollination_probability(method, distance)
    ├─ competition_growth(N, r, K, M, alpha)
    └─ shannon_diversity(proportions)
```

## Dependency Stack

```text
vanaspati
├── hisab      — math primitives (vectors, geometry)
├── serde      — serialization
├── thiserror  — error derive
└── tracing    — structured logging
```

## Consumers

| Crate     | What it uses                                           |
|-----------|--------------------------------------------------------|
| kiran     | Growth models + seasons for vegetation in game worlds  |
| joshua    | Full ecosystem simulation, agricultural modeling       |
| garjan    | Seasonal state for foliage sound synthesis             |
| jantu     | Pollination + ecosystem for creature habitat/food      |

## Design Principles

- **Pure functions** — no mutable global state, no side effects beyond tracing
- **f32 throughout** — game/simulation precision, not scientific double precision
- **Factory presets** — quick start with realistic defaults (oak, bamboo, grass)
- **Feature-gated logging** — tracing subscriber is opt-in via `logging` feature
- **Serializable** — all types derive `Serialize`/`Deserialize` for save/load
