use vanaspati::{
    AllocationStrategy, DispersalMethod, GrowthModel, PhotosynthesisPathway, PollinationMethod,
    RootSystem, Season, age_mortality_rate, allocate, competition_growth, daylight_hours_at,
    dispersal_distance, dispersal_probability, drought_mortality, frost_mortality,
    growth_modifier_at, height_to_diameter, height_to_leaf_area, net_primary_productivity,
    pathway_params, photosynthesis_rate, pollination_probability, self_thinning_mortality,
    shannon_diversity, temperature_factor, temperature_factor_c4,
};

fn main() {
    // --- Growth ---
    let oak = GrowthModel::oak();
    let bamboo = GrowthModel::bamboo();
    println!("=== Growth (1 year) ===");
    println!("  Oak:    {:.2} m", oak.height_at_day(365.0));
    println!("  Bamboo: {:.2} m", bamboo.height_at_day(365.0));

    // --- Photosynthesis (C3 vs C4) ---
    let (_, c3_a, c3_p) = pathway_params(PhotosynthesisPathway::C3);
    let (_, c4_a, c4_p) = pathway_params(PhotosynthesisPathway::C4);
    println!("\n=== Photosynthesis at 35°C, PAR=1000 ===");
    let c3 = photosynthesis_rate(c3_p, c3_a, 1000.0) * temperature_factor(35.0, 25.0);
    let c4 = photosynthesis_rate(c4_p, c4_a, 1000.0) * temperature_factor_c4(35.0);
    println!("  C3: {c3:.2} µmol CO₂/m²/s");
    println!("  C4: {c4:.2} µmol CO₂/m²/s");

    // --- Seasons (latitude-aware) ---
    println!("\n=== Daylight hours (summer solstice) ===");
    for lat in [0.0, 30.0, 45.0, 60.0] {
        let h = daylight_hours_at(172, lat);
        let m = growth_modifier_at(172, lat);
        println!("  {lat:>4.0}°N: {h:.1}h daylight, growth modifier {m:.2}");
    }

    // --- Southern hemisphere ---
    let south_jan = Season::from_day_latitude(15, -35.0);
    let north_jan = Season::from_day_latitude(15, 35.0);
    println!("\n=== Hemisphere comparison (Jan 15) ===");
    println!("  35°N: {north_jan:?}");
    println!("  35°S: {south_jan:?}");

    // --- Root systems ---
    let oak_roots = RootSystem::oak();
    println!("\n=== Root Systems ===");
    println!(
        "  Oak: {:.1}m deep, stabilization {:.1}",
        oak_roots.max_depth_m,
        oak_roots.stabilization_factor()
    );

    // --- Pollination + Dispersal pipeline ---
    let poll = pollination_probability(PollinationMethod::Insect, 100.0);
    let disp = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 5.0);
    let prob = dispersal_probability(DispersalMethod::Wind, disp * 0.5);
    println!("\n=== Reproduction pipeline ===");
    println!("  Pollination at 100m (insect): {poll:.2}");
    println!("  Max wind dispersal: {disp:.1}m");
    println!("  Seed landing at {:.1}m: {prob:.3}", disp * 0.5);

    // --- Biomass allocation ---
    let pool = allocate(1000.0, AllocationStrategy::Balanced);
    let height = oak.height_at_day(365.0);
    let diam = height_to_diameter(height, 0.04);
    let la = height_to_leaf_area(height, 8.0);
    println!("\n=== Biomass (1000g carbon, balanced) ===");
    println!(
        "  Leaf: {:.3} kg, Stem: {:.3} kg, Root: {:.3} kg",
        pool.leaf_kg, pool.stem_kg, pool.root_kg
    );
    println!("  Oak at {height:.2}m → diameter {diam:.3}m, leaf area {la:.1}m²");

    // --- Mortality ---
    let age_mort = age_mortality_rate(30000.0, 36500.0);
    let frost = frost_mortality(-15.0, -10.0);
    let drought = drought_mortality(30.0, 100.0);
    let thinning = self_thinning_mortality(10.0, 1.0);
    println!("\n=== Mortality rates ===");
    println!("  Age (82yr oak):     {age_mort:.6}/day");
    println!("  Frost (-15°C, -10 hardiness): {frost:.3}");
    println!("  Drought (30/100 water):       {drought:.3}");
    println!("  Self-thinning (10/m², 1kg):   {thinning:.3}");

    // --- Ecosystem ---
    let diversity = shannon_diversity(&[0.4, 0.3, 0.2, 0.1]);
    let npp = net_primary_productivity(1200.0, 500.0);
    let growth = competition_growth(100.0, 0.1, 1000.0, 50.0, 0.5);
    println!("\n=== Ecosystem ===");
    println!("  Shannon diversity (4 species): {diversity:.3}");
    println!("  NPP: {npp:.0} g C/m²/year");
    println!("  Competition growth: {growth:.2}");
}
