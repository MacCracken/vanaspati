use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vanaspati::{
    AllocationStrategy, DispersalMethod, GrowthModel, PhotosynthesisPathway, PollinationMethod,
    RootSystem, Season, age_mortality_rate, allocate, canopy_to_habitat_score, competition_growth,
    daylight_hours_at, dispersal_distance, dispersal_probability, drought_mortality,
    frost_mortality, frost_risk_to_mortality, growing_conditions_to_growth_multiplier,
    growth_modifier_at, growth_stage, height_to_diameter, height_to_leaf_area,
    net_primary_productivity, pathway_params, photosynthesis_rate, pollination_probability,
    self_thinning_mortality, shannon_diversity, soil_temperature_to_root_activity, solar_to_par,
    temperature_factor, temperature_factor_c4, temperature_factor_cam, wind_to_dispersal_speed,
};

fn bench_growth(c: &mut Criterion) {
    let oak = GrowthModel::oak();
    let bamboo = GrowthModel::bamboo();

    c.bench_function("growth_oak_height_at_day", |b| {
        b.iter(|| oak.height_at_day(black_box(365.0)))
    });
    c.bench_function("growth_bamboo_height_at_day", |b| {
        b.iter(|| bamboo.height_at_day(black_box(365.0)))
    });
    c.bench_function("growth_oak_daily_growth", |b| {
        b.iter(|| oak.daily_growth(black_box(5.0)))
    });
    c.bench_function("growth_stage", |b| {
        b.iter(|| growth_stage(black_box(12.5), black_box(25.0)))
    });
}

fn bench_photosynthesis(c: &mut Criterion) {
    c.bench_function("photosynthesis_rate", |b| {
        b.iter(|| photosynthesis_rate(black_box(20.0), black_box(0.05), black_box(800.0)))
    });
    c.bench_function("temperature_factor", |b| {
        b.iter(|| temperature_factor(black_box(25.0), black_box(25.0)))
    });
    c.bench_function("temperature_factor_c4", |b| {
        b.iter(|| temperature_factor_c4(black_box(32.0)))
    });
    c.bench_function("temperature_factor_cam", |b| {
        b.iter(|| temperature_factor_cam(black_box(28.0)))
    });
    c.bench_function("pathway_params", |b| {
        b.iter(|| pathway_params(black_box(PhotosynthesisPathway::C4)))
    });
    c.bench_function("canopy_light_at_depth", |b| {
        b.iter(|| {
            vanaspati::canopy_light_at_depth(black_box(1000.0), black_box(3.0), black_box(0.5))
        })
    });
    c.bench_function("shaded_photosynthesis_rate", |b| {
        b.iter(|| {
            vanaspati::shaded_photosynthesis_rate(
                black_box(20.0),
                black_box(0.05),
                black_box(1000.0),
                black_box(3.0),
                black_box(0.5),
            )
        })
    });
}

fn bench_season(c: &mut Criterion) {
    c.bench_function("season_from_day", |b| {
        b.iter(|| Season::from_day(black_box(200)))
    });
    c.bench_function("season_growth_modifier", |b| {
        b.iter(|| Season::Summer.growth_modifier())
    });
    c.bench_function("season_daylight_hours", |b| {
        b.iter(|| Season::Summer.daylight_hours())
    });
    c.bench_function("daylight_hours_at_45n", |b| {
        b.iter(|| daylight_hours_at(black_box(172), black_box(45.0)))
    });
    c.bench_function("daylight_hours_at_equator", |b| {
        b.iter(|| daylight_hours_at(black_box(172), black_box(0.0)))
    });
    c.bench_function("growth_modifier_at_45n", |b| {
        b.iter(|| growth_modifier_at(black_box(172), black_box(45.0)))
    });
    c.bench_function("from_day_latitude_south", |b| {
        b.iter(|| Season::from_day_latitude(black_box(15), black_box(-35.0)))
    });
}

fn bench_root(c: &mut Criterion) {
    let oak = RootSystem::oak();
    c.bench_function("root_stabilization_factor", |b| {
        b.iter(|| oak.stabilization_factor())
    });
}

fn bench_pollination(c: &mut Criterion) {
    c.bench_function("pollination_probability_insect", |b| {
        b.iter(|| pollination_probability(black_box(PollinationMethod::Insect), black_box(250.0)))
    });
    c.bench_function("pollination_probability_self", |b| {
        b.iter(|| {
            pollination_probability(
                black_box(PollinationMethod::SelfPollinating),
                black_box(0.0),
            )
        })
    });
}

fn bench_dispersal(c: &mut Criterion) {
    c.bench_function("dispersal_distance_wind", |b| {
        b.iter(|| {
            dispersal_distance(
                black_box(DispersalMethod::Wind),
                black_box(0.001),
                black_box(10.0),
                black_box(5.0),
            )
        })
    });
    c.bench_function("dispersal_distance_gravity", |b| {
        b.iter(|| {
            dispersal_distance(
                black_box(DispersalMethod::Gravity),
                black_box(6.0),
                black_box(25.0),
                black_box(0.0),
            )
        })
    });
    c.bench_function("dispersal_probability_wind", |b| {
        b.iter(|| dispersal_probability(black_box(DispersalMethod::Wind), black_box(50.0)))
    });
    c.bench_function("dispersal_probability_animal", |b| {
        b.iter(|| dispersal_probability(black_box(DispersalMethod::Animal), black_box(200.0)))
    });
}

fn bench_biomass(c: &mut Criterion) {
    c.bench_function("allocate_balanced", |b| {
        b.iter(|| allocate(black_box(1000.0), black_box(AllocationStrategy::Balanced)))
    });
    c.bench_function("allocate_stressed_root", |b| {
        b.iter(|| {
            allocate(
                black_box(1000.0),
                black_box(AllocationStrategy::StressedRoot),
            )
        })
    });
    c.bench_function("height_to_diameter", |b| {
        b.iter(|| height_to_diameter(black_box(25.0), black_box(0.04)))
    });
    c.bench_function("height_to_leaf_area", |b| {
        b.iter(|| height_to_leaf_area(black_box(25.0), black_box(8.0)))
    });
}

fn bench_mortality(c: &mut Criterion) {
    c.bench_function("age_mortality_rate", |b| {
        b.iter(|| age_mortality_rate(black_box(10000.0), black_box(36500.0)))
    });
    c.bench_function("self_thinning_mortality", |b| {
        b.iter(|| self_thinning_mortality(black_box(5.0), black_box(1.0)))
    });
    c.bench_function("frost_mortality", |b| {
        b.iter(|| frost_mortality(black_box(-15.0), black_box(-10.0)))
    });
    c.bench_function("drought_mortality", |b| {
        b.iter(|| drought_mortality(black_box(50.0), black_box(100.0)))
    });
}

fn bench_ecosystem(c: &mut Criterion) {
    c.bench_function("competition_growth", |b| {
        b.iter(|| {
            competition_growth(
                black_box(100.0),
                black_box(0.1),
                black_box(1000.0),
                black_box(50.0),
                black_box(0.5),
            )
        })
    });

    let proportions = vec![0.25, 0.25, 0.25, 0.25];
    c.bench_function("shannon_diversity_4sp", |b| {
        b.iter(|| shannon_diversity(black_box(&proportions)))
    });

    let proportions_20 = vec![0.05; 20];
    c.bench_function("shannon_diversity_20sp", |b| {
        b.iter(|| shannon_diversity(black_box(&proportions_20)))
    });

    c.bench_function("net_primary_productivity", |b| {
        b.iter(|| net_primary_productivity(black_box(1200.0), black_box(500.0)))
    });
}

fn bench_bridge(c: &mut Criterion) {
    c.bench_function("bridge_solar_to_par", |b| {
        b.iter(|| solar_to_par(black_box(800.0)))
    });
    c.bench_function("bridge_growing_conditions", |b| {
        b.iter(|| {
            growing_conditions_to_growth_multiplier(
                black_box(25.0),
                black_box(25.0),
                black_box(800.0),
                black_box(172),
                black_box(45.0),
            )
        })
    });
    c.bench_function("bridge_soil_root_activity", |b| {
        b.iter(|| soil_temperature_to_root_activity(black_box(293.15)))
    });
    c.bench_function("bridge_frost_risk_mortality", |b| {
        b.iter(|| frost_risk_to_mortality(black_box(-5.0), black_box(0.8), black_box(-10.0)))
    });
    c.bench_function("bridge_canopy_habitat", |b| {
        b.iter(|| canopy_to_habitat_score(black_box(4.0)))
    });
    c.bench_function("bridge_wind_dispersal", |b| {
        b.iter(|| {
            wind_to_dispersal_speed(
                black_box(10.0),
                black_box(10.0),
                black_box(20.0),
                black_box(1.0),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_growth,
    bench_photosynthesis,
    bench_season,
    bench_root,
    bench_pollination,
    bench_dispersal,
    bench_biomass,
    bench_mortality,
    bench_ecosystem,
    bench_bridge,
);
criterion_main!(benches);
