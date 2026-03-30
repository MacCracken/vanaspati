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

- **Growth** — logistic growth curves with stage progression (seed through senescence), factory presets (oak, bamboo, grass), water & nitrogen stress factors
- **Photosynthesis** — C3/C4/CAM pathways, light response curves, temperature factors, Beer-Lambert canopy light competition, water stress
- **Water** — soil water storage (5 soil types), precipitation/infiltration/runoff, drainage, transpiration, evaporation, daily water balance
- **Nitrogen** — soil N pools (available + organic), mineralization, plant uptake, leaching, nitrogen stress on growth
- **Stomata** — Ball-Berry conductance, VPD, transpiration, drought/VPD factors, boundary layer conductance
- **Seasons** — latitude-parameterized daylight hours (sunrise equation), hemisphere-aware season mapping, growth modifiers
- **Phenology** — growing degree days, chilling hours (Utah model), phenological events (bud break through dormancy)
- **Root Systems** — taproot, fibrous, adventitious — depth, spread, water uptake, root zone fraction, stabilization
- **Biomass** — organ pools (leaf/stem/root/reproductive), allocation strategies, allometric scaling
- **Mortality** — age (Weibull), drought, frost, competition (self-thinning), fire, disease, windthrow
- **Dispersal** — wind, animal, gravity, water, explosive — exponential decay kernel, seed profiles
- **Pollination** — wind, insect, bird, water, self — distance-based probability
- **Decomposition** — litter types, Q10 temperature/moisture factors, exponential decay, nitrogen release
- **Herbivory** — grazing/browsing/frugivory/root feeding, compensatory growth, herbivory mortality
- **Succession** — pioneer/climax dynamics, shade tolerance, establishment probability, competitive displacement
- **Reproduction** — vegetative methods (runners, rhizomes, root sprouts, layering), clonal spread, resource-limited ramet production
- **Fire Ecology** — fire adaptation strategies (resprouter, thick-barked, serotinous), bark protection, post-fire regeneration
- **Mycorrhiza** — ectomycorrhizal/arbuscular/ericoid types, nutrient enhancement, colonization, hyphal networks
- **Allelopathy** — allelochemical production, soil accumulation/decay, dose-response growth/germination inhibition
- **Ecosystem** — Lotka-Volterra competition, Shannon-Wiener diversity, net primary productivity
- **Bridge** — 20+ cross-crate functions connecting badal (weather), ushma (thermodynamics), pravash (fluid), jantu (creatures)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vanaspati = "1.0"
```

```rust
use vanaspati::{GrowthModel, Season, photosynthesis_rate, SoilWater, daily_water_balance};

// Grow an oak tree
let oak = GrowthModel::oak();
let height = oak.height_at_day(365.0);    // meters after 1 year
let daily = oak.daily_growth(5.0);        // meters/day at 5m tall

// Photosynthesis: Pmax=20 µmol CO₂/m²/s, α=0.05, PAR=800 µmol/m²/s
let rate = photosynthesis_rate(20.0, 0.05, 800.0);

// Seasonal growth modifier
let modifier = Season::Summer.growth_modifier(); // 1.0

// Soil water balance
let mut soil = SoilWater::loam();
let fluxes = daily_water_balance(&mut soil, 10.0, 3.0, 2.0);
```

### Optional Features

```toml
[dependencies]
vanaspati = { version = "1.0", features = ["logging"] }
```

| Feature        | Description                                  |
|----------------|----------------------------------------------|
| `logging`      | Structured logging via `tracing`             |
| `soorat-compat`| Visualization types for soorat/kiran rendering |

## Dependencies

| Crate       | Purpose                              |
|-------------|--------------------------------------|
| `hisab`     | Math primitives (interpolation, transforms, numerics) |
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
