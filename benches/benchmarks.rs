use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vanaspati::{
    AllocationStrategy, DispersalMethod, FireStrategy, GrowthModel, HerbivoryType, LeafHabit,
    LitterType, MycorrhizalType, PftParams, PftType, PhenologicalEvent, PhotosynthesisPathway,
    PollinationMethod, RootSystem, Season, SoilCarbon, SoilNitrogen, SoilType, SoilWater,
    SuccessionalStage, VegetativeMethod, accumulated_gdd, age_mortality_rate, allocate,
    ball_berry_conductance, bark_protection, biomass_removal, canopy_to_habitat_score,
    clonal_area_m2, co2_factor, colonization_rate, compensatory_growth_factor, competition_growth,
    daily_decomposition_rate, daily_nitrogen_balance, daily_som_turnover, daily_water_balance,
    daylight_hours_at, disease_mortality, dispersal_distance, dispersal_probability,
    drought_mortality, effective_growth_multiplier, effective_lai, enhanced_n_uptake,
    establishment_probability, event_reached, fire_mortality, frost_mortality,
    frost_risk_to_mortality, growing_conditions_to_growth_multiplier, growing_degree_days,
    growth_inhibition, growth_modifier_at, growth_respiration, growth_stage, height_to_diameter,
    height_to_leaf_area, herbivory_mortality, infiltration_rate, lai_from_biomass,
    maintenance_respiration, mineralization_rate, net_primary_productivity,
    net_primary_productivity_carbon, nitrogen_release, nitrogen_stress_factor, nitrogen_uptake,
    pathway_params, penman_monteith_et, phenological_progress, photosynthesis_rate,
    pollination_probability, reference_et, remaining_mass, resource_limited_ramets, resprout_vigor,
    saturated_conductivity, seasonal_lai_multiplier, self_thinning_mortality, serotinous_release,
    shannon_diversity, soil_concentration, soil_evaporation, soil_temperature_to_root_activity,
    solar_to_par, surface_resistance, temperature_factor, temperature_factor_c4,
    temperature_factor_cam, total_leaf_conductance, transpiration_rate, vapor_pressure_deficit,
    water_stress_factor, water_stress_growth_factor, wind_to_dispersal_speed, windthrow_mortality,
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
    c.bench_function("root_water_uptake_mm", |b| {
        let oak = RootSystem::oak();
        let soil = SoilWater::loam();
        b.iter(|| oak.water_uptake_mm(black_box(&soil), black_box(5.0)))
    });
    c.bench_function("water_stress_factor", |b| {
        b.iter(|| water_stress_factor(black_box(0.5)))
    });
    c.bench_function("water_stress_growth_factor", |b| {
        b.iter(|| water_stress_growth_factor(black_box(0.5)))
    });
}

fn bench_nitrogen(c: &mut Criterion) {
    c.bench_function("mineralization_rate", |b| {
        b.iter(|| mineralization_rate(black_box(0.5), black_box(25.0), black_box(0.6)))
    });
    c.bench_function("nitrogen_uptake", |b| {
        b.iter(|| {
            nitrogen_uptake(
                black_box(0.001),
                black_box(0.01),
                black_box(500.0),
                black_box(0.8),
            )
        })
    });
    c.bench_function("nitrogen_stress_factor", |b| {
        b.iter(|| nitrogen_stress_factor(black_box(0.010), black_box(0.012)))
    });
    c.bench_function("daily_nitrogen_balance", |b| {
        let mut sn = SoilNitrogen::forest();
        b.iter(|| {
            sn = SoilNitrogen::forest(); // reset
            daily_nitrogen_balance(
                &mut sn,
                black_box(20.0),
                black_box(0.6),
                black_box(0.0005),
                black_box(200.0),
                black_box(10.0),
                black_box(200.0),
            )
        })
    });
}

fn bench_herbivory(c: &mut Criterion) {
    c.bench_function("biomass_removal_grazing", |b| {
        b.iter(|| {
            biomass_removal(
                black_box(50.0),
                black_box(100.0),
                black_box(30.0),
                black_box(10.0),
                black_box(HerbivoryType::Grazing),
                black_box(0.5),
            )
        })
    });
    c.bench_function("compensatory_growth_factor", |b| {
        b.iter(|| compensatory_growth_factor(black_box(0.3), black_box(0.3)))
    });
    c.bench_function("herbivory_mortality", |b| {
        b.iter(|| herbivory_mortality(black_box(0.85), black_box(0.5)))
    });
}

fn bench_succession(c: &mut Criterion) {
    c.bench_function("establishment_probability", |b| {
        b.iter(|| establishment_probability(black_box(0.5), black_box(SuccessionalStage::Pioneer)))
    });
    c.bench_function("effective_growth_multiplier", |b| {
        b.iter(|| effective_growth_multiplier(black_box(0.5), black_box(SuccessionalStage::Climax)))
    });
}

fn bench_reproduction(c: &mut Criterion) {
    c.bench_function("resource_limited_ramets", |b| {
        b.iter(|| {
            resource_limited_ramets(
                black_box(VegetativeMethod::Rhizome),
                black_box(0.8),
                black_box(0.7),
            )
        })
    });
    c.bench_function("clonal_area_m2", |b| {
        b.iter(|| clonal_area_m2(black_box(VegetativeMethod::Rhizome), black_box(5.0)))
    });
}

fn bench_fire(c: &mut Criterion) {
    c.bench_function("fire_mortality", |b| {
        b.iter(|| fire_mortality(black_box(0.6), black_box(0.5)))
    });
    c.bench_function("bark_protection", |b| {
        b.iter(|| bark_protection(black_box(FireStrategy::ThickBarked)))
    });
    c.bench_function("resprout_vigor", |b| {
        b.iter(|| resprout_vigor(black_box(FireStrategy::Resprouter), black_box(0.5)))
    });
    c.bench_function("serotinous_release", |b| {
        b.iter(|| {
            serotinous_release(
                black_box(FireStrategy::Serotinous),
                black_box(1000.0),
                black_box(0.7),
            )
        })
    });
}

fn bench_mycorrhiza(c: &mut Criterion) {
    c.bench_function("enhanced_n_uptake", |b| {
        b.iter(|| {
            enhanced_n_uptake(
                black_box(0.001),
                black_box(MycorrhizalType::Ectomycorrhizal),
                black_box(0.7),
            )
        })
    });
    c.bench_function("colonization_rate", |b| {
        b.iter(|| colonization_rate(black_box(MycorrhizalType::Arbuscular), black_box(0.3)))
    });
}

fn bench_allelopathy(c: &mut Criterion) {
    c.bench_function("soil_concentration", |b| {
        b.iter(|| {
            soil_concentration(
                black_box(0.5),
                black_box(0.01),
                black_box(25.0),
                black_box(0.6),
            )
        })
    });
    c.bench_function("growth_inhibition", |b| {
        b.iter(|| growth_inhibition(black_box(0.5), black_box(5.0)))
    });
}

fn bench_mortality_ext(c: &mut Criterion) {
    c.bench_function("disease_mortality", |b| {
        b.iter(|| disease_mortality(black_box(0.5)))
    });
    c.bench_function("windthrow_mortality", |b| {
        b.iter(|| windthrow_mortality(black_box(25.0), black_box(30.0), black_box(0.5)))
    });
}

fn bench_respiration(c: &mut Criterion) {
    c.bench_function("maintenance_respiration", |b| {
        b.iter(|| maintenance_respiration(black_box(2000.0), black_box(0.004), black_box(25.0)))
    });
    c.bench_function("growth_respiration", |b| {
        b.iter(|| growth_respiration(black_box(0.5)))
    });
    c.bench_function("net_primary_productivity_carbon", |b| {
        b.iter(|| net_primary_productivity_carbon(black_box(0.10), black_box(0.04)))
    });
}

fn bench_lai(c: &mut Criterion) {
    c.bench_function("lai_from_biomass", |b| {
        b.iter(|| lai_from_biomass(black_box(50.0), black_box(25.0), black_box(100.0)))
    });
    c.bench_function("seasonal_lai_multiplier", |b| {
        b.iter(|| {
            seasonal_lai_multiplier(
                black_box(LeafHabit::Deciduous),
                black_box(200),
                black_box(45.0),
            )
        })
    });
    c.bench_function("effective_lai", |b| {
        b.iter(|| {
            effective_lai(
                black_box(6.0),
                black_box(8.0),
                black_box(0.9),
                black_box(1.0),
                black_box(0.0),
            )
        })
    });
}

fn bench_et(c: &mut Criterion) {
    c.bench_function("penman_monteith_et", |b| {
        b.iter(|| {
            penman_monteith_et(
                black_box(15.0),
                black_box(1.5),
                black_box(25.0),
                black_box(1.5),
                black_box(2.0),
                black_box(70.0),
                black_box(101.3),
            )
        })
    });
    c.bench_function("reference_et", |b| {
        b.iter(|| {
            reference_et(
                black_box(15.0),
                black_box(25.0),
                black_box(1.5),
                black_box(2.0),
            )
        })
    });
    c.bench_function("surface_resistance", |b| {
        b.iter(|| surface_resistance(black_box(0.25), black_box(4.0)))
    });
}

fn bench_co2_som(c: &mut Criterion) {
    c.bench_function("co2_factor_c3", |b| {
        b.iter(|| co2_factor(black_box(560.0), black_box(PhotosynthesisPathway::C3)))
    });
    c.bench_function("daily_som_turnover", |b| {
        let mut sc = SoilCarbon::temperate_forest();
        b.iter(|| {
            sc = SoilCarbon::temperate_forest();
            daily_som_turnover(&mut sc, black_box(0.005), black_box(20.0), black_box(0.5))
        })
    });
}

fn bench_pft(c: &mut Criterion) {
    c.bench_function("pft_from_type", |b| {
        b.iter(|| PftParams::from_type(black_box(PftType::TemperateBroadleafDeciduous)))
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
    bench_nitrogen,
    bench_herbivory,
    bench_succession,
    bench_reproduction,
    bench_fire,
    bench_mycorrhiza,
    bench_allelopathy,
    bench_mortality_ext,
    bench_respiration,
    bench_lai,
    bench_et,
    bench_co2_som,
    bench_pft,
);
criterion_main!(benches);
