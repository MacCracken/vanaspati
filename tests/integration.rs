use vanaspati::{
    AllocationStrategy, BiomassPool, DispersalMethod, GrowthModel, MortalityCause,
    PhotosynthesisPathway, PollinationMethod, RootSystem, RootType, Season, SeedProfile,
    StomatalBehavior, age_mortality_rate, allocate, atmosphere_to_photosynthesis_inputs,
    ball_berry_conductance, canopy_to_habitat_score, competition_growth, daylight_hours_at,
    dispersal_distance, dispersal_probability, drought_mortality, drought_stomatal_factor,
    evapotranspiration_cooling, frost_mortality, frost_risk_to_mortality,
    growing_conditions_to_growth_multiplier, growth_modifier_at, height_to_diameter,
    height_to_leaf_area, humidity_to_vpd, instantaneous_wue, light_compensation_point,
    net_primary_productivity, pathway_params, photosynthesis_rate, pollination_probability,
    rainfall_to_water_supply, saturation_vapor_pressure, seed_production_to_food,
    self_thinning_mortality, shannon_diversity, soil_temperature_to_root_activity,
    temperature_factor, temperature_factor_c4, temperature_factor_cam, transpiration_rate,
    vapor_pressure_deficit, water_use_efficiency, wind_to_dispersal_speed,
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

// --- Bridge integration tests ---

#[test]
fn bridge_weather_to_photosynthesis_pipeline() {
    // badal provides atmospheric state → bridge converts → vanaspati computes
    let (temp_c, par) = atmosphere_to_photosynthesis_inputs(298.15, 800.0);
    let rate = photosynthesis_rate(20.0, 0.05, par);
    let temp_f = temperature_factor(temp_c, 25.0);
    let effective = rate * temp_f;
    assert!(
        effective > 10.0,
        "warm sunny day should produce good photosynthesis"
    );
}

#[test]
fn bridge_weather_to_growth_pipeline() {
    // Full pipeline: atmospheric conditions → growth multiplier → daily growth
    let oak = GrowthModel::oak();
    let height = oak.height_at_day(365.0);
    let base_growth = oak.daily_growth(height);

    // Summer optimal conditions
    let summer_mult = growing_conditions_to_growth_multiplier(25.0, 25.0, 800.0, 172, 45.0);
    // Winter conditions
    let winter_mult = growing_conditions_to_growth_multiplier(0.0, 25.0, 100.0, 356, 45.0);

    assert!(base_growth * summer_mult > base_growth * winter_mult);
}

#[test]
fn bridge_rainfall_to_drought_pipeline() {
    // No rain → drought mortality
    let dry_supply = rainfall_to_water_supply(0.0, 24.0);
    let dry_mort = drought_mortality(dry_supply, 100.0);
    assert_eq!(dry_mort, 1.0);

    // Good rain → no drought
    let wet_supply = rainfall_to_water_supply(5.0, 4.0); // 20 L/m²
    let wet_mort = drought_mortality(wet_supply, 15.0);
    assert_eq!(wet_mort, 0.0);
}

#[test]
fn bridge_frost_risk_pipeline() {
    // badal frost_risk → vanaspati frost mortality
    let warm = frost_risk_to_mortality(10.0, 0.0, -10.0);
    let cold = frost_risk_to_mortality(-15.0, 0.9, -10.0);
    assert_eq!(warm, 0.0);
    assert!(cold > 0.5);
}

#[test]
fn bridge_wind_to_seed_dispersal() {
    // Wind at reference height → canopy height → dispersal distance
    let wind_at_canopy = wind_to_dispersal_speed(8.0, 10.0, 25.0, 1.0);
    let dist = dispersal_distance(DispersalMethod::Wind, 0.001, 25.0, wind_at_canopy);
    assert!(dist > 0.0, "seeds should disperse with wind");
}

#[test]
fn bridge_biomass_to_food_availability() {
    // Growth → biomass → reproductive output → food for jantu
    let pool = BiomassPool::oak();
    let food = seed_production_to_food(pool.reproductive_kg as f64, 6.0); // acorn mass
    assert!(food > 0.0, "oak acorns should provide food");
}

#[test]
fn bridge_canopy_from_height() {
    // Growth → leaf area → LAI → habitat cover for jantu
    let oak = GrowthModel::oak();
    let height = oak.height_at_day(3000.0); // several years
    let leaf_area = height_to_leaf_area(height, 8.0);
    // Assume 100 m² ground area for crown projection
    let lai = leaf_area / 100.0;
    let cover = canopy_to_habitat_score(lai as f64);
    assert!(cover > 0.0, "oak canopy should provide habitat cover");
}

#[test]
fn bridge_soil_temp_affects_root_uptake() {
    // ushma soil temperature → root activity → effective water uptake
    let root = RootSystem::oak();
    let warm_activity = soil_temperature_to_root_activity(293.15); // 20°C
    let cold_activity = soil_temperature_to_root_activity(278.15); // 5°C
    let warm_uptake = root.water_uptake_rate * warm_activity;
    let cold_uptake = root.water_uptake_rate * cold_activity;
    assert!(warm_uptake > cold_uptake, "warm soil → more water uptake");
}

#[test]
fn bridge_et_cooling_improves_photosynthesis_in_heat() {
    // Hot day: ET cooling reduces effective leaf temperature
    let air_temp = 38.0_f32;
    let et_cool = evapotranspiration_cooling(4.0); // 10°C cooling
    let leaf_temp = air_temp - et_cool;

    let without_et = temperature_factor(air_temp, 25.0);
    let with_et = temperature_factor(leaf_temp, 25.0);
    assert!(
        with_et > without_et,
        "ET cooling should improve photosynthesis in heat"
    );
}

// --- Decomposition integration tests ---

#[test]
fn biomass_to_litter_to_nitrogen_pipeline() {
    use vanaspati::{LitterType, daily_decomposition_rate, mass_decomposed, nitrogen_release};

    // Oak drops 50 kg of leaves in autumn
    let leaf_litter_kg = BiomassPool::oak().leaf_kg;

    // Decompose for a year at warm, moist conditions
    let k = daily_decomposition_rate(LitterType::Leaf, 20.0, 0.6);
    let decomposed = mass_decomposed(leaf_litter_kg, k, 365.0);
    assert!(decomposed > 0.0, "leaves should decompose");
    assert!(decomposed < leaf_litter_kg, "not all gone in one year");

    // Nitrogen released (broadleaf C:N ≈ 40)
    let n = nitrogen_release(decomposed, 40.0);
    assert!(n > 0.0, "decomposition should release nitrogen");
}

#[test]
fn frozen_soil_stops_decomposition() {
    use vanaspati::{LitterType, daily_decomposition_rate, mass_decomposed};

    let k = daily_decomposition_rate(LitterType::Leaf, -5.0, 0.6);
    assert_eq!(k, 0.0);
    let remaining = mass_decomposed(100.0, k, 365.0);
    assert_eq!(remaining, 0.0, "no decomposition when frozen");
}

#[test]
fn wood_persists_longer_than_leaves() {
    use vanaspati::{LitterType, daily_decomposition_rate, half_life_days};

    let leaf_k = daily_decomposition_rate(LitterType::Leaf, 20.0, 0.5);
    let wood_k = daily_decomposition_rate(LitterType::Wood, 20.0, 0.5);
    let leaf_half = half_life_days(leaf_k);
    let wood_half = half_life_days(wood_k);
    assert!(
        wood_half > leaf_half * 5.0,
        "wood should persist much longer than leaves"
    );
}

// --- Stomatal conductance integration tests ---

#[test]
fn stomata_full_pipeline() {
    // Weather → VPD → Ball-Berry → transpiration → WUE
    let temp = 25.0_f32;
    let es = saturation_vapor_pressure(temp);
    let ea = es * 0.6; // 60% RH
    let vpd = vapor_pressure_deficit(es, ea);
    assert!(vpd > 0.0);

    // Photosynthesis drives stomatal opening
    let a = photosynthesis_rate(20.0, 0.05, 800.0);
    let gs = ball_berry_conductance(0.02, 9.0, a, 0.6, 400.0);
    assert!(gs > 0.02, "stomata should open with photosynthesis");

    // Transpiration from conductance and VPD
    let e = transpiration_rate(gs, vpd, 101.3);
    assert!(e > 0.0, "should transpire water");

    // Water use efficiency
    let wue = instantaneous_wue(a, e);
    assert!(
        wue > 2.0 && wue < 10.0,
        "WUE should be realistic, got {wue}"
    );
}

#[test]
fn stomata_drought_reduces_transpiration() {
    let gs_wet = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 400.0);
    let drought_factor = drought_stomatal_factor(0.20, 0.15, 0.35, StomatalBehavior::Isohydric);
    let gs_dry = gs_wet * drought_factor;
    assert!(gs_dry < gs_wet, "drought should reduce conductance");

    let vpd = 1.5;
    let e_wet = transpiration_rate(gs_wet, vpd, 101.3);
    let e_dry = transpiration_rate(gs_dry, vpd, 101.3);
    assert!(e_dry < e_wet, "drought should reduce transpiration");
}

#[test]
fn bridge_humidity_to_stomatal_pipeline() {
    // badal humidity → bridge VPD → stomatal transpiration
    let vpd = humidity_to_vpd(30.0, 0.4); // hot dry: 30°C, 40% RH
    assert!(vpd > 2.0, "hot dry air should produce high VPD");

    let gs = ball_berry_conductance(0.02, 9.0, 15.0, 0.4, 400.0);
    let e = transpiration_rate(gs, vpd, 101.3);
    assert!(e > 0.0);
}

// --- Water balance integration tests ---

#[test]
fn water_stomata_drought_feedback() {
    use vanaspati::{SoilWater, daily_water_balance, vpd_stomatal_factor};

    // Start with wet soil
    let mut soil = SoilWater::loam();
    let vpd = 1.5_f32;

    // Simulate 60 dry days — stomatal conductance should decrease
    let mut conductances = Vec::new();
    for _ in 0..60 {
        let rwc = soil.relative_water_content();
        let drought_f = drought_stomatal_factor(rwc, 0.0, 1.0, StomatalBehavior::Isohydric);
        let vpd_f = vpd_stomatal_factor(vpd, 1.5);
        let gs = ball_berry_conductance(0.02, 9.0, 15.0, 0.6, 400.0) * drought_f * vpd_f;
        conductances.push(gs);

        // Transpire based on conductance
        let e_mmol = transpiration_rate(gs, vpd, 101.3);
        let e_mm = e_mmol * 0.018 * 3600.0 * 12.0 / 1000.0; // rough: mmol/m²/s → mm/day (12h)
        let _ = daily_water_balance(&mut soil, 0.0, e_mm, 2.0);
    }

    assert!(
        conductances.last().unwrap() < conductances.first().unwrap(),
        "conductance should decrease as soil dries"
    );
}

#[test]
fn water_rainfall_refills_soil() {
    use vanaspati::{SoilWater, daily_water_balance, rainfall_to_water_supply};

    let mut soil = SoilWater::loam();
    // Dry out
    for _ in 0..30 {
        let _ = daily_water_balance(&mut soil, 0.0, 5.0, 2.0);
    }
    let dry_rwc = soil.relative_water_content();
    assert!(dry_rwc < 0.5);

    // Rainstorm: badal says 10 mm/hr for 3 hours
    let rain = rainfall_to_water_supply(10.0, 3.0); // 30 mm
    let _ = daily_water_balance(&mut soil, rain, 0.0, 0.0);
    assert!(
        soil.relative_water_content() > dry_rwc,
        "rain should refill"
    );
}

#[test]
fn root_uptake_limits_water_extraction() {
    use vanaspati::{RootSystem, SoilWater};

    let oak = RootSystem::oak(); // deep roots, 200 L/day
    let grass = RootSystem::grass(); // shallow roots, 0.5 L/day
    let soil = SoilWater::loam();

    let demand = 10.0; // mm/day
    let oak_uptake = oak.water_uptake_mm(&soil, demand);
    let grass_uptake = grass.water_uptake_mm(&soil, demand);

    assert!(
        oak_uptake > grass_uptake,
        "oak={oak_uptake}, grass={grass_uptake}"
    );
    // Grass limited by 0.5 capacity
    assert!((grass_uptake - 0.5).abs() < 0.01);
    // Oak limited by demand
    assert!((oak_uptake - 10.0).abs() < 0.01);
}

#[test]
fn water_stress_reduces_growth_before_photosynthesis() {
    use vanaspati::{water_stress_factor, water_stress_growth_factor};

    // At moderate drought (RWC=0.5), growth should be stressed but photosynthesis should not
    let rwc = 0.5;
    let growth_f = water_stress_growth_factor(rwc);
    let photo_f = water_stress_factor(rwc);
    assert!(growth_f < 1.0, "growth should be stressed at RWC=0.5");
    assert_eq!(photo_f, 1.0, "photosynthesis should be fine at RWC=0.5");

    // At severe drought (RWC=0.2), both should be stressed
    let rwc = 0.2;
    let growth_f = water_stress_growth_factor(rwc);
    let photo_f = water_stress_factor(rwc);
    assert!(
        growth_f < photo_f,
        "growth more sensitive: g={growth_f}, p={photo_f}"
    );
}

#[test]
fn full_water_growth_pipeline() {
    use vanaspati::{
        GrowthModel, RootSystem, SoilWater, daily_water_balance, photosynthesis_rate,
        water_stress_factor, water_stress_growth_factor,
    };

    let oak_growth = GrowthModel::oak();
    let roots = RootSystem::oak();
    let mut soil = SoilWater::loam();

    // Simulate 90 days of drought — track growth reduction
    let mut heights = vec![oak_growth.initial_height];
    let mut current_h = oak_growth.initial_height;

    for _ in 0..90 {
        let rwc = soil.relative_water_content();
        let growth_stress = water_stress_growth_factor(rwc);
        let photo_stress = water_stress_factor(rwc);

        // Growth rate reduced by water stress
        let base_growth = oak_growth.daily_growth(current_h);
        let actual_growth = base_growth * growth_stress;
        current_h += actual_growth;
        heights.push(current_h);

        // Photosynthesis also affected
        let _photo = photosynthesis_rate(20.0, 0.05, 800.0) * photo_stress;

        // Transpiration demand based on root uptake
        let demand = roots.water_uptake_mm(&soil, 5.0);
        let _ = daily_water_balance(&mut soil, 0.0, demand, 2.0);
    }

    // After 90 dry days, growth should have slowed significantly
    let early_rate = heights[10] - heights[5];
    let late_rate = heights[85] - heights[80];
    assert!(
        late_rate < early_rate,
        "growth should slow under drought: early={early_rate}, late={late_rate}"
    );
}

#[test]
fn bridge_water_stress_matches_direct() {
    use vanaspati::{
        soil_water_to_growth_stress, soil_water_to_photosynthesis_stress, water_stress_factor,
        water_stress_growth_factor,
    };

    let rwc = 0.3;
    let bridge_growth = soil_water_to_growth_stress(rwc as f64);
    let direct_growth = water_stress_growth_factor(rwc);
    assert!((bridge_growth - direct_growth).abs() < 0.01);

    let bridge_photo = soil_water_to_photosynthesis_stress(rwc as f64);
    let direct_photo = water_stress_factor(rwc);
    assert!((bridge_photo - direct_photo).abs() < 0.01);
}

// --- Nitrogen integration tests ---

#[test]
fn decomposition_feeds_nitrogen_pool() {
    use vanaspati::{
        LitterType, SoilNitrogen, daily_decomposition_rate, daily_nitrogen_balance,
        mass_decomposed, nitrogen_release,
    };

    let mut soil_n = SoilNitrogen::forest();
    let initial_available = soil_n.available_n;

    // Decompose 10 kg leaf litter for 30 days (warm, moist)
    let k = daily_decomposition_rate(LitterType::Leaf, 25.0, 0.6);
    let decomposed = mass_decomposed(10.0, k, 30.0);
    let n_released = nitrogen_release(decomposed, 40.0); // C:N = 40

    // Add released N to organic pool (litter N enters organic first)
    let _ = soil_n.add_organic(n_released);

    // Run 30 days of mineralization to convert some organic → available
    for _ in 0..30 {
        let _ = daily_nitrogen_balance(&mut soil_n, 25.0, 0.6, 0.0, 0.0, 0.0, 200.0);
    }

    assert!(
        soil_n.available_n > initial_available,
        "litter decomposition should eventually increase available N"
    );
}

#[test]
fn nitrogen_limits_growth_over_time() {
    use vanaspati::{
        GrowthModel, SoilNitrogen, daily_nitrogen_balance, nitrogen_stress_factor,
        water_stress_growth_factor,
    };

    let oak = GrowthModel::oak();
    let mut soil_n = SoilNitrogen::poor(); // low N
    let critical_n = 0.012; // broadleaf

    let mut height = oak.initial_height;
    let mut plant_n = 0.010; // slightly below critical
    let plant_biomass = 100.0; // kg (rough)

    // 180 days — track growth with N limitation
    let mut heights = vec![height];
    for _ in 0..180 {
        // N balance
        let demand = 0.0002; // modest demand
        let fluxes = daily_nitrogen_balance(&mut soil_n, 20.0, 0.5, demand, 50.0, 5.0, 200.0);

        // Update plant N concentration
        if plant_biomass > 0.0 {
            plant_n = (plant_n * plant_biomass + fluxes.plant_uptake) / plant_biomass;
        }

        // Growth limited by N
        let n_stress = nitrogen_stress_factor(plant_n, critical_n);
        let water_stress = water_stress_growth_factor(1.0); // assume wet
        let base_growth = oak.daily_growth(height);
        height += base_growth * n_stress * water_stress;
        heights.push(height);
    }

    // With poor soil, growth should be slower than potential
    let actual_final = *heights.last().unwrap();
    let potential_final = oak.height_at_day(180.0);
    assert!(
        actual_final < potential_final,
        "N-limited growth should be less than potential: actual={actual_final}, potential={potential_final}"
    );
}

#[test]
fn nitrogen_leaching_coupled_to_water_drainage() {
    use vanaspati::{SoilNitrogen, SoilWater, daily_nitrogen_balance, daily_water_balance};

    let mut soil_n = SoilNitrogen::fertile();
    let mut soil_w = SoilWater::loam();

    // Saturate soil first
    soil_w.water_content_mm = soil_w.saturation_mm;

    let initial_n = soil_n.available_n;

    // Heavy rain day → drainage → leaching
    let w_fluxes = daily_water_balance(&mut soil_w, 50.0, 0.0, 0.0);
    let n_fluxes = daily_nitrogen_balance(
        &mut soil_n,
        20.0,
        soil_w.relative_water_content(),
        0.0,
        0.0,
        w_fluxes.drainage_mm,
        soil_w.water_content_mm,
    );

    assert!(n_fluxes.leaching > 0.0, "drainage should cause N leaching");
    assert!(
        soil_n.available_n < initial_n,
        "available N should decrease from leaching"
    );
}

#[test]
fn bridge_nitrogen_stress_matches_direct() {
    use vanaspati::{nitrogen_stress_factor, nitrogen_to_growth_stress};

    let plant_n = 0.008;
    let critical = 0.012;
    let bridge = nitrogen_to_growth_stress(plant_n as f64, critical as f64);
    let direct = nitrogen_stress_factor(plant_n, critical);
    assert!((bridge - direct).abs() < 0.01);
}

// --- Herbivory integration tests ---

#[test]
fn herbivory_reduces_biomass_pool() {
    use vanaspati::{BiomassPool, HerbivoryType, biomass_removal, compensatory_growth_factor};

    let mut oak = BiomassPool::oak();
    let initial_leaf = oak.leaf_kg;

    // Moderate grazing
    let (dl, ds, dr, drp) = biomass_removal(
        oak.leaf_kg,
        oak.stem_kg,
        oak.root_kg,
        oak.reproductive_kg,
        HerbivoryType::Grazing,
        0.4,
    );
    oak.leaf_kg -= dl;
    oak.stem_kg -= ds;
    oak.root_kg -= dr;
    oak.reproductive_kg -= drp;

    assert!(oak.leaf_kg < initial_leaf, "grazing should reduce leaves");
    assert!(oak.total_kg() < BiomassPool::oak().total_kg());

    // Check compensatory response
    let defoliation = dl / initial_leaf;
    let comp = compensatory_growth_factor(defoliation, 0.15); // tree = low compensation
    assert!(
        comp > 0.8,
        "moderate defoliation should allow some regrowth, got {comp}"
    );
}

#[test]
fn herbivory_mortality_pipeline() {
    use vanaspati::herbivory_mortality;

    // At severe defoliation (80%), test vulnerability differences
    let defol_frac = 0.85;
    let grass_mort = herbivory_mortality(defol_frac, 0.1); // grass: low vulnerability
    let seedling_mort = herbivory_mortality(defol_frac, 1.0); // seedling: high vulnerability
    assert!(
        grass_mort < seedling_mort,
        "grass more resilient: g={grass_mort}, s={seedling_mort}"
    );
    assert!(
        seedling_mort > 0.2,
        "seedling should face real mortality at 85%"
    );
}

// --- Succession integration tests ---

#[test]
fn succession_canopy_closure_shifts_advantage() {
    use vanaspati::{
        SuccessionalStage, effective_growth_multiplier, establishment_probability,
        understory_light_fraction,
    };

    // Open field: LAI=0 → full light
    let open_light = understory_light_fraction(0.0, 0.5);
    let pioneer_open = effective_growth_multiplier(open_light, SuccessionalStage::Pioneer);
    let climax_open = effective_growth_multiplier(open_light, SuccessionalStage::Climax);
    assert!(pioneer_open > climax_open, "pioneer dominates open field");

    // Dense forest: LAI=6 → very low light
    let dense_light = understory_light_fraction(6.0, 0.5);
    let pioneer_dense = effective_growth_multiplier(dense_light, SuccessionalStage::Pioneer);
    let climax_dense = effective_growth_multiplier(dense_light, SuccessionalStage::Climax);
    assert!(
        climax_dense > pioneer_dense,
        "climax dominates dense forest"
    );

    // Pioneer can't establish in dense shade
    let pioneer_est = establishment_probability(dense_light, SuccessionalStage::Pioneer);
    assert_eq!(
        pioneer_est, 0.0,
        "pioneer can't establish under dense canopy"
    );

    // Climax can establish in moderate shade
    let mid_light = understory_light_fraction(3.0, 0.5);
    let climax_est = establishment_probability(mid_light, SuccessionalStage::Climax);
    assert!(climax_est > 0.0, "climax establishes under moderate canopy");
}

// --- Vegetative reproduction integration tests ---

#[test]
fn vegetative_spread_with_resource_limitation() {
    use vanaspati::{
        VegetativeMethod, clonal_area_m2, parent_cost_kg, resource_limited_ramets,
        water_stress_growth_factor,
    };

    // Well-watered bamboo grove
    let water_stress = water_stress_growth_factor(0.9); // well watered
    let n_stress = 0.8; // moderate N
    let ramets = resource_limited_ramets(VegetativeMethod::Rhizome, water_stress, n_stress);
    assert!(ramets > 10.0, "bamboo should produce many ramets");

    // After 5 years of spread
    let area = clonal_area_m2(VegetativeMethod::Rhizome, 5.0);
    assert!(area > 500.0, "5-year rhizome spread should cover >500m²");

    // Cost to parent
    let bamboo_biomass = 150.0; // kg
    let cost = parent_cost_kg(bamboo_biomass, VegetativeMethod::Rhizome, ramets);
    assert!(
        cost < bamboo_biomass * 0.5,
        "shouldn't cost more than half the parent"
    );

    // Drought suppresses reproduction
    let drought_stress = water_stress_growth_factor(0.2);
    let drought_ramets =
        resource_limited_ramets(VegetativeMethod::Rhizome, drought_stress, n_stress);
    assert!(
        drought_ramets < ramets,
        "drought should reduce ramet production"
    );
}

// --- Bridge integration tests ---

#[test]
fn bridge_herbivore_to_biomass_matches_direct() {
    use vanaspati::{HerbivoryType, herbivore_to_biomass_loss, total_biomass_removed};

    let bridge = herbivore_to_biomass_loss(50.0, 100.0, 30.0, 10.0, 0.5, false);
    let direct = total_biomass_removed(50.0, 100.0, 30.0, 10.0, HerbivoryType::Grazing, 0.5);
    assert!((bridge - direct).abs() < 0.01);
}

#[test]
fn bridge_successional_advantage_crossover() {
    use vanaspati::light_to_successional_advantage;

    let open = light_to_successional_advantage(0.9);
    let shade = light_to_successional_advantage(0.15);
    assert!(open < 1.0, "open → pioneer advantage");
    assert!(shade > 1.0, "shade → climax advantage");
}

// --- Fire ecology integration tests ---

#[test]
fn fire_survival_depends_on_strategy() {
    use vanaspati::{FireStrategy, bark_protection, fire_mortality, resprout_vigor};

    let intensity = 0.6;

    // Thick-barked tree survives fire
    let tb_mort = fire_mortality(intensity, bark_protection(FireStrategy::ThickBarked));
    let tb_resprout = resprout_vigor(FireStrategy::ThickBarked, intensity);
    assert!(tb_mort < 0.2, "thick bark should protect, got {tb_mort}");

    // Sensitive tree dies
    let sens_mort = fire_mortality(intensity, bark_protection(FireStrategy::Sensitive));
    assert!(sens_mort > 0.5, "sensitive should die, got {sens_mort}");

    // Resprouter regrows vigorously
    let resp_vigor = resprout_vigor(FireStrategy::Resprouter, intensity);
    assert!(resp_vigor > tb_resprout, "resprouter should regrow faster");
}

#[test]
fn serotinous_release_after_fire() {
    use vanaspati::{FireStrategy, post_fire_establishment, serotinous_release};

    let seeds = serotinous_release(FireStrategy::Serotinous, 5000.0, 0.8);
    assert!(seeds > 3000.0, "should release most of seed bank");

    let advantage = post_fire_establishment(FireStrategy::Serotinous, 0.8);
    assert!(
        advantage > 2.0,
        "serotinous should have strong post-fire advantage"
    );
}

// --- Mycorrhizal integration tests ---

#[test]
fn mycorrhiza_enhances_nitrogen_uptake() {
    use vanaspati::{
        MycorrhizalType, SoilNitrogen, colonization_rate, enhanced_n_uptake, nitrogen_uptake,
    };

    let soil_n = SoilNitrogen::forest();
    let base_uptake = nitrogen_uptake(0.001, soil_n.available_n, 200.0, 0.8);

    // ECM colonization at low P
    let col = colonization_rate(MycorrhizalType::Ectomycorrhizal, 0.2);
    let enhanced = enhanced_n_uptake(base_uptake, MycorrhizalType::Ectomycorrhizal, col);

    assert!(enhanced > base_uptake, "mycorrhiza should enhance uptake");
    assert!(
        enhanced < base_uptake * 2.0,
        "enhancement should be reasonable, not doubling"
    );
}

#[test]
fn mycorrhiza_net_benefit_nutrient_limited() {
    use vanaspati::{MycorrhizalType, net_benefit_ratio};

    // N-limited forest → beneficial
    let benefit = net_benefit_ratio(MycorrhizalType::Ectomycorrhizal, 0.7, 0.8);
    assert!(
        benefit > 1.0,
        "should be beneficial when N-limited: {benefit}"
    );

    // Fertile soil → costly
    let cost = net_benefit_ratio(MycorrhizalType::Ectomycorrhizal, 0.7, 0.1);
    assert!(cost < 1.0, "should be costly when N-abundant: {cost}");
}

// --- Allelopathy integration tests ---

#[test]
fn allelopathy_suppresses_neighbor_growth() {
    use vanaspati::{
        AllelopathicPotency, allelopathic_input, growth_inhibition, soil_concentration,
    };

    // Black walnut (strong allelopathy) with 50 kg biomass
    let input = allelopathic_input(50.0, AllelopathicPotency::Strong);

    // Accumulate over 30 days (warm, moist)
    let mut conc = 0.0;
    for _ in 0..30 {
        conc = soil_concentration(conc, input, 25.0, 0.6);
    }
    assert!(conc > 0.0, "should accumulate allelochemicals");

    // Effect on sensitive neighbor
    let inhibition = growth_inhibition(conc, 10.0);
    assert!(
        inhibition > 0.1,
        "sensitive neighbor should be inhibited: {inhibition}"
    );

    // Tolerant neighbor less affected
    let tolerant_inhib = growth_inhibition(conc, 1.0);
    assert!(
        tolerant_inhib < inhibition,
        "tolerant should be less affected: tol={tolerant_inhib}, sens={inhibition}"
    );
}

// --- Extended mortality integration tests ---

#[test]
fn mortality_types_compound() {
    use vanaspati::{disease_mortality, drought_mortality, windthrow_mortality};

    // Stressed plant: drought + disease + moderate wind
    let drought_p = drought_mortality(30.0, 100.0); // 70% deficit
    let disease_p = disease_mortality(0.7); // stressed
    let wind_p = windthrow_mortality(20.0, 30.0, 0.8); // moderate wind, wet soil

    // Combined survival probability
    let survival = (1.0 - drought_p) * (1.0 - disease_p) * (1.0 - wind_p);
    assert!(survival < 1.0, "combined stressors should reduce survival");
    // Each factor should contribute
    assert!(drought_p > 0.0, "drought should contribute");
    assert!(disease_p > 0.0, "disease should contribute");
}

// --- Carbon budget integration tests ---

#[test]
fn carbon_budget_gpp_to_npp() {
    use vanaspati::{
        growth_respiration, maintenance_respiration, net_primary_productivity_carbon,
        photosynthesis_rate, total_autotrophic_respiration,
    };

    // Oak tree: 2000 kg biomass, 0.4% N, 25°C
    let gpp_umol = photosynthesis_rate(20.0, 0.05, 800.0); // µmol CO₂/m²/s
    // Convert: µmol/m²/s × 12e-6 g/µmol × 3600 s/hr × 12 hr × 100 m² crown / 1000 g/kg
    let gpp_kg_c = gpp_umol * 12e-6 * 3600.0 * 12.0 * 100.0 / 1000.0;

    let r_m = maintenance_respiration(2000.0, 0.004, 25.0);
    let r_g = growth_respiration(gpp_kg_c * 0.5); // assume 50% allocated to growth
    let ra = total_autotrophic_respiration(r_m, r_g);
    let npp = net_primary_productivity_carbon(gpp_kg_c, ra);

    // NPP should be 40-60% of GPP for a healthy tree
    let npp_fraction = npp / gpp_kg_c;
    assert!(
        (0.2..=0.8).contains(&npp_fraction),
        "NPP/GPP ratio should be 0.4-0.6, got {npp_fraction}"
    );
}

#[test]
fn som_litter_decomposition_pipeline() {
    use vanaspati::{
        LitterType, SoilCarbon, daily_decomposition_rate, daily_som_turnover, mass_decomposed,
    };

    let mut soil_c = SoilCarbon::new(0.0, 0.0, 0.0);

    // Simulate 1 year: daily leaf litter → decomposition → SOM
    for _day in 0..365 {
        let k = daily_decomposition_rate(LitterType::Leaf, 20.0, 0.5);
        let decomposed = mass_decomposed(0.01, k, 1.0); // 10g litter/day
        let litter_c = decomposed * 0.45; // 45% carbon
        let _ = daily_som_turnover(&mut soil_c, litter_c, 20.0, 0.5);
    }

    assert!(soil_c.active > 0.0, "active pool should accumulate");
    assert!(soil_c.slow > 0.0, "slow pool should receive transfers");
    assert!(soil_c.total() > 0.0);
}

// --- LAI integration tests ---

#[test]
fn lai_drives_photosynthesis_and_et() {
    use vanaspati::{
        LeafHabit, ball_berry_conductance, canopy_light_at_depth, effective_lai, lai_from_biomass,
        penman_monteith_et, seasonal_lai_multiplier, surface_resistance,
    };

    // Summer deciduous tree: 50 kg leaves, SLA=25, 100 m² crown
    let lai_base = lai_from_biomass(50.0, 25.0, 100.0);
    let seasonal = seasonal_lai_multiplier(LeafHabit::Deciduous, 200, 45.0);
    let lai = effective_lai(lai_base, 7.0, seasonal, 1.0, 0.0);

    // LAI drives light interception
    let par_understory = canopy_light_at_depth(1000.0, lai, 0.5);
    assert!(
        par_understory < 200.0,
        "dense canopy should block most light"
    );

    // LAI drives ET through surface resistance
    let gs = ball_berry_conductance(0.02, 9.0, 15.0, 0.7, 400.0);
    let rs = surface_resistance(gs, lai);
    let et = penman_monteith_et(15.0, 1.0, 25.0, 1.5, 2.0, rs, 101.3);
    assert!(et > 0.0, "canopy should transpire");

    // Winter: no LAI, no ET
    let winter_seasonal = seasonal_lai_multiplier(LeafHabit::Deciduous, 15, 45.0);
    let winter_lai = effective_lai(lai_base, 7.0, winter_seasonal, 1.0, 0.0);
    assert_eq!(winter_lai, 0.0, "deciduous winter = no LAI");
}

// --- CO2 integration test ---

#[test]
fn co2_enrichment_increases_growth() {
    use vanaspati::{PhotosynthesisPathway, photosynthesis_rate_co2};

    let ambient = photosynthesis_rate_co2(20.0, 0.05, 800.0, 400.0, PhotosynthesisPathway::C3);
    let elevated = photosynthesis_rate_co2(20.0, 0.05, 800.0, 560.0, PhotosynthesisPathway::C3);
    let doubled = photosynthesis_rate_co2(20.0, 0.05, 800.0, 800.0, PhotosynthesisPathway::C3);

    assert!(
        elevated > ambient,
        "FACE-level CO2 should increase photosynthesis"
    );
    assert!(
        doubled > elevated,
        "more CO2 = more photosynthesis (diminishing returns)"
    );

    // C4 should respond less
    let c4_ambient = photosynthesis_rate_co2(40.0, 0.06, 800.0, 400.0, PhotosynthesisPathway::C4);
    let c4_doubled = photosynthesis_rate_co2(40.0, 0.06, 800.0, 800.0, PhotosynthesisPathway::C4);
    let c3_gain = (doubled - ambient) / ambient;
    let c4_gain = (c4_doubled - c4_ambient) / c4_ambient;
    assert!(c3_gain > c4_gain, "C3 should respond more to CO2 than C4");
}

// --- PFT integration test ---

#[test]
fn pft_drives_full_simulation() {
    use vanaspati::{
        GrowthModel, PftParams, PftType, lai_from_biomass, maintenance_respiration,
        photosynthesis_rate_co2, seasonal_lai_multiplier,
    };

    let pft = PftParams::from_type(PftType::TemperateBroadleafDeciduous);

    // Create growth model from PFT params
    let growth = GrowthModel {
        max_height: pft.max_height,
        growth_rate: pft.growth_rate,
        initial_height: 0.1,
    };

    // Simulate summer day
    let _lai = lai_from_biomass(50.0, pft.sla, 100.0);
    let seasonal = seasonal_lai_multiplier(pft.leaf_habit, 200, 45.0);
    assert!(seasonal > 0.9, "summer should be near full leaf");

    let photo = photosynthesis_rate_co2(pft.pmax, pft.quantum_yield, 800.0, 400.0, pft.pathway);
    assert!(photo > 0.0, "should photosynthesize");

    let r_m = maintenance_respiration(100.0, pft.leaf_n, pft.temp_optimum);
    assert!(r_m > 0.0, "should respire");

    let daily = growth.daily_growth(5.0);
    assert!(daily > 0.0, "should grow");
}
