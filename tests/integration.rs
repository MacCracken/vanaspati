use vanaspati::{
    AllocationStrategy, BiomassPool, DispersalMethod, GrowthModel, MortalityCause,
    PhotosynthesisPathway, PollinationMethod, RootSystem, RootType, Season, SeedProfile,
    age_mortality_rate, allocate, competition_growth, daylight_hours_at, dispersal_distance,
    dispersal_probability, drought_mortality, frost_mortality, growth_modifier_at,
    height_to_diameter, height_to_leaf_area, light_compensation_point, net_primary_productivity,
    pathway_params, photosynthesis_rate, pollination_probability, self_thinning_mortality,
    shannon_diversity, temperature_factor, temperature_factor_c4, temperature_factor_cam,
    water_use_efficiency,
};

// --- V0.1.0 integration tests ---

#[test]
fn growth_model_oak_preset() {
    let oak = GrowthModel::oak();
    let h0 = oak.height_at_day(0.0);
    let h365 = oak.height_at_day(365.0);
    assert!(h365 > h0, "oak should grow over a year");
    assert!(
        h365 < oak.max_height,
        "oak should not reach max in one year"
    );
}

#[test]
fn seasonal_growth_modifiers_sum_reasonable() {
    let seasons = [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ];
    let total: f32 = seasons.iter().map(|s| s.growth_modifier()).sum();
    assert!(total > 0.0);
    assert!(total < 4.0);
}

#[test]
fn photosynthesis_combined_with_temperature() {
    let base_rate = photosynthesis_rate(20.0, 0.05, 800.0);
    let temp_opt = temperature_factor(25.0, 25.0);
    let temp_cold = temperature_factor(5.0, 25.0);
    assert!(base_rate * temp_opt > base_rate * temp_cold);
}

#[test]
fn ecosystem_dynamics_consistent() {
    let equal = shannon_diversity(&[0.25, 0.25, 0.25, 0.25]);
    let dominated = shannon_diversity(&[0.91, 0.03, 0.03, 0.03]);
    assert!(equal > dominated);
    assert_eq!(net_primary_productivity(800.0, 300.0), 500.0);
    assert!(competition_growth(100.0, 0.5, 1000.0, 0.0, 0.0) > 0.0);
}

// --- V0.2.0 integration tests ---

#[test]
fn all_re_exports_accessible() {
    // V0.1.0
    let _ = GrowthModel::oak();
    let _ = Season::Spring;
    let _ = RootSystem::oak();
    let _ = RootType::Taproot;
    let _ = PollinationMethod::Wind;
    let _ = photosynthesis_rate(20.0, 0.05, 800.0);
    let _ = light_compensation_point(2.0, 0.05);
    let _ = water_use_efficiency(6.0, 3.0);
    let _ = temperature_factor(25.0, 25.0);
    let _ = pollination_probability(PollinationMethod::Wind, 100.0);
    let _ = competition_growth(50.0, 0.1, 1000.0, 0.0, 0.5);
    let _ = shannon_diversity(&[0.5, 0.5]);
    let _ = net_primary_productivity(1000.0, 400.0);
    // V0.2.0
    let _ = DispersalMethod::Wind;
    let _ = SeedProfile::dandelion();
    let _ = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 5.0);
    let _ = dispersal_probability(DispersalMethod::Wind, 50.0);
    let _ = BiomassPool::oak();
    let _ = AllocationStrategy::Balanced;
    let _ = allocate(1000.0, AllocationStrategy::Balanced);
    let _ = height_to_diameter(25.0, 0.04);
    let _ = height_to_leaf_area(25.0, 8.0);
    let _ = MortalityCause::Age;
    let _ = age_mortality_rate(100.0, 36500.0);
    let _ = self_thinning_mortality(5.0, 1.0);
    let _ = frost_mortality(-15.0, -10.0);
    let _ = drought_mortality(50.0, 100.0);
    let _ = PhotosynthesisPathway::C3;
    let _ = pathway_params(PhotosynthesisPathway::C4);
    let _ = temperature_factor_c4(32.0);
    let _ = temperature_factor_cam(28.0);
    let _ = daylight_hours_at(172, 45.0);
    let _ = growth_modifier_at(172, 45.0);
    let _ = Season::from_day_latitude(15, -35.0);
}

#[test]
fn pollination_to_dispersal_pipeline() {
    // Plant pollinates, then disperses seeds
    let poll_prob = pollination_probability(PollinationMethod::Insect, 100.0);
    assert!(poll_prob > 0.0, "pollination should succeed at 100m");

    // If pollinated, seeds disperse
    let seed_dist = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 5.0);
    assert!(seed_dist > 0.0, "wind dispersal should produce distance");

    let seed_prob = dispersal_probability(DispersalMethod::Wind, seed_dist * 0.5);
    assert!(
        seed_prob > 0.0,
        "some seeds should land at half max distance"
    );
}

#[test]
fn growth_to_biomass_pipeline() {
    // Grow an oak for a year, then compute allometrics
    let oak = GrowthModel::oak();
    let height = oak.height_at_day(365.0);
    assert!(height > 0.0);

    let diameter = height_to_diameter(height, 0.04);
    let leaf_area = height_to_leaf_area(height, 8.0);
    assert!(diameter > 0.0);
    assert!(leaf_area > 0.0);

    // Allocate daily carbon gain
    let daily_growth_m = oak.daily_growth(height);
    let carbon_g = daily_growth_m * 500.0; // rough conversion
    let pool = allocate(carbon_g, AllocationStrategy::Balanced);
    assert!(pool.total_kg() > 0.0);
}

#[test]
fn c4_vs_c3_at_warm_temp() {
    let (_, c3_alpha, c3_pmax) = pathway_params(PhotosynthesisPathway::C3);
    let (_, c4_alpha, c4_pmax) = pathway_params(PhotosynthesisPathway::C4);

    let warm_temp = 35.0;
    let par = 1000.0;

    let c3_rate = photosynthesis_rate(c3_pmax, c3_alpha, par) * temperature_factor(warm_temp, 25.0);
    let c4_rate = photosynthesis_rate(c4_pmax, c4_alpha, par) * temperature_factor_c4(warm_temp);

    assert!(
        c4_rate > c3_rate,
        "C4 should outperform C3 at warm temperatures"
    );
}

#[test]
fn latitude_daylight_drives_photosynthesis() {
    let tropical_daylight = daylight_hours_at(172, 10.0); // near equator
    let arctic_daylight = daylight_hours_at(172, 65.0); // near arctic

    // Arctic gets more summer daylight (long days)
    assert!(arctic_daylight > tropical_daylight);

    // But tropical is more consistent year-round
    let tropical_winter = daylight_hours_at(356, 10.0);
    let arctic_winter = daylight_hours_at(356, 65.0);
    assert!(tropical_winter > arctic_winter);
}

#[test]
fn mortality_interacts_with_ecosystem() {
    // Dense population should trigger self-thinning
    let thinning = self_thinning_mortality(10.0, 1.0);
    assert!(thinning > 0.0, "dense stand should thin");

    // Frost kills frost-sensitive species
    let tropical_frost = frost_mortality(-5.0, 0.0);
    let hardy_frost = frost_mortality(-5.0, -25.0);
    assert!(
        tropical_frost > hardy_frost,
        "tropical plant should die first"
    );
}
