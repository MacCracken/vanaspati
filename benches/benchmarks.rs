use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vanaspati::{
    AllocationStrategy, DispersalMethod, GrowthModel, LitterType, PhenologicalEvent,
    PhotosynthesisPathway, PollinationMethod, RootSystem, Season, SoilType, SoilWater,
    accumulated_gdd, age_mortality_rate, allocate, ball_berry_conductance, canopy_to_habitat_score,
    competition_growth, daily_decomposition_rate, daily_water_balance, daylight_hours_at,
    dispersal_distance, dispersal_probability, drought_mortality, event_reached, frost_mortality,
    frost_risk_to_mortality, growing_conditions_to_growth_multiplier, growing_degree_days,
    growth_modifier_at, growth_stage, height_to_diameter, height_to_leaf_area, infiltration_rate,
    net_primary_productivity, nitrogen_release, pathway_params, phenological_progress,
    photosynthesis_rate, pollination_probability, remaining_mass, saturated_conductivity,
    self_thinning_mortality, shannon_diversity, soil_evaporation,
    soil_temperature_to_root_activity, solar_to_par, temperature_factor, temperature_factor_c4,
    temperature_factor_cam, total_leaf_conductance, transpiration_rate, vapor_pressure_deficit,
    wind_to_dispersal_speed,
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

fn bench_decomposition(c: &mut Criterion) {
    c.bench_function("daily_decomposition_rate", |b| {
        b.iter(|| {
            daily_decomposition_rate(black_box(LitterType::Leaf), black_box(20.0), black_box(0.6))
        })
    });
    c.bench_function("remaining_mass", |b| {
        b.iter(|| remaining_mass(black_box(100.0), black_box(0.004), black_box(365.0)))
    });
    c.bench_function("nitrogen_release", |b| {
        b.iter(|| nitrogen_release(black_box(10.0), black_box(40.0)))
    });
}

fn bench_phenology(c: &mut Criterion) {
    c.bench_function("growing_degree_days", |b| {
        b.iter(|| growing_degree_days(black_box(20.0), black_box(5.0)))
    });
    let temps: Vec<f32> = (0..365)
        .map(|d| 10.0 + 15.0 * (d as f32 * 0.0172).sin())
        .collect();
    c.bench_function("accumulated_gdd_365", |b| {
        b.iter(|| accumulated_gdd(black_box(&temps), black_box(5.0)))
    });
    c.bench_function("event_reached", |b| {
        b.iter(|| event_reached(black_box(400.0), black_box(PhenologicalEvent::LeafOut)))
    });
    c.bench_function("phenological_progress", |b| {
        b.iter(|| phenological_progress(black_box(250.0), black_box(PhenologicalEvent::Flowering)))
    });
}

fn bench_stomata(c: &mut Criterion) {
    c.bench_function("ball_berry_conductance", |b| {
        b.iter(|| {
            ball_berry_conductance(
                black_box(0.02),
                black_box(9.0),
                black_box(15.0),
                black_box(0.7),
                black_box(400.0),
            )
        })
    });
    c.bench_function("transpiration_rate", |b| {
        b.iter(|| transpiration_rate(black_box(0.25), black_box(1.5), black_box(101.3)))
    });
    c.bench_function("vapor_pressure_deficit", |b| {
        b.iter(|| vapor_pressure_deficit(black_box(2.338), black_box(1.5)))
    });
    c.bench_function("total_leaf_conductance", |b| {
        b.iter(|| total_leaf_conductance(black_box(0.25), black_box(0.9)))
    });
}

fn bench_water(c: &mut Criterion) {
    c.bench_function("soil_water_new_loam", |b| {
        b.iter(|| SoilWater::new(black_box(SoilType::Loam), black_box(1.0)))
    });
    c.bench_function("saturated_conductivity", |b| {
        b.iter(|| saturated_conductivity(black_box(SoilType::Loam)))
    });
    c.bench_function("infiltration_rate", |b| {
        b.iter(|| {
            infiltration_rate(
                black_box(SoilType::Loam),
                black_box(100.0),
                black_box(500.0),
                black_box(20.0),
            )
        })
    });
    c.bench_function("soil_evaporation", |b| {
        b.iter(|| soil_evaporation(black_box(5.0), black_box(200.0), black_box(270.0)))
    });
    c.bench_function("daily_water_balance", |b| {
        let mut soil = SoilWater::loam();
        b.iter(|| {
            soil.water_content_mm = soil.field_capacity_mm; // reset each iter
            daily_water_balance(&mut soil, black_box(10.0), black_box(3.0), black_box(2.0))
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
    bench_decomposition,
    bench_phenology,
    bench_stomata,
    bench_water,
);
criterion_main!(benches);
