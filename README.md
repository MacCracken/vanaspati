# Vanaspati

**Vanaspati** (वनस्पति — Sanskrit for "lord of the forest, plant/tree") — botany and plant behavior engine for growth, photosynthesis, seasonal cycles, and ecosystems.

Part of the [AGNOS](https://github.com/MacCracken) ecosystem.

## What It Does

Vanaspati defines the **science of plants** — growth models, photosynthesis, seasonal cycles, root systems, pollination, and ecosystem dynamics. It provides the math; consumers decide what to do with it (render forests, simulate ecosystems, model agriculture).

```text
kiran     → vegetation in game worlds (forests, grass, crops)
vanaspati → defines plant behavior (growth, photosynthesis, seasons)  ← this crate
joshua    → ecosystem simulation, agricultural modeling
badal     → weather/climate data drives growth
jantu     → animals interact with plants (habitat, food)
garjan    → foliage rustling, branch creaking
```

## Features

- **Growth** — logistic growth curves with stage progression (seed through senescence), factory presets (oak, bamboo, grass)
- **Photosynthesis** — light response curves, compensation point, water use efficiency, temperature factor (C3 bell curve)
- **Seasons** — day-of-year mapping, daylight hours, growth modifiers (northern hemisphere)
- **Root Systems** — taproot, fibrous, adventitious — with depth, spread, water uptake, stabilization
- **Pollination** — wind, insect, bird, water, self-pollinating — distance-based probability
- **Ecosystem** — Lotka-Volterra competition, Shannon-Wiener diversity, net primary productivity

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vanaspati = "0.1"
```

```rust
use vanaspati::{GrowthModel, Season, photosynthesis_rate};

// Grow an oak tree
let oak = GrowthModel::oak();
let height = oak.height_at_day(365.0);    // meters after 1 year
let daily = oak.daily_growth(5.0);        // meters/day at 5m tall

// Photosynthesis: Pmax=20 µmol CO₂/m²/s, α=0.05, PAR=800 µmol/m²/s
let rate = photosynthesis_rate(20.0, 0.05, 800.0);

// Seasonal growth modifier
let modifier = Season::Summer.growth_modifier(); // 1.0
```

### Optional Features

```toml
[dependencies]
vanaspati = { version = "0.1", features = ["logging"] }
```

| Feature   | Description                          |
|-----------|--------------------------------------|
| `logging` | Structured logging via `tracing`     |

## Dependencies

| Crate       | Purpose                              |
|-------------|--------------------------------------|
| `hisab`     | Math primitives (vectors, geometry)  |
| `serde`     | Serialization / deserialization      |
| `thiserror` | Error handling                       |
| `tracing`   | Structured logging                   |

## Consumers

| Crate       | Usage                                              |
|-------------|----------------------------------------------------|
| **kiran**   | Vegetation in game worlds (forests, grass, crops)   |
| **joshua**  | Ecosystem simulation, agricultural modeling         |
| **garjan**  | Foliage rustling, branch creaking                   |
| **jantu**   | Habitat/food source for creature behavior           |

## Development

```bash
make check      # fmt + clippy + test + audit
make test       # cargo test --all-features
make bench      # run benchmarks with history
make coverage   # HTML coverage report
make doc        # build docs (warnings = errors)
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow.

## Roadmap

See [docs/development/roadmap.md](docs/development/roadmap.md) for completed work, planned cross-crate bridges, and future features.

## License

[GPL-3.0](LICENSE)
