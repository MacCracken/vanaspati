use vanaspati::{
    GrowthModel, PollinationMethod, RootSystem, Season, competition_growth,
    net_primary_productivity, photosynthesis_rate, pollination_probability, shannon_diversity,
    temperature_factor,
};

fn main() {
    // --- Growth ---
    let oak = GrowthModel::oak();
    let bamboo = GrowthModel::bamboo();
    println!("=== Growth (1 year) ===");
    println!("  Oak:    {:.2} m", oak.height_at_day(365.0));
    println!("  Bamboo: {:.2} m", bamboo.height_at_day(365.0));

    // --- Photosynthesis ---
    // Pmax = 20 µmol CO₂/m²/s, quantum yield = 0.05, PAR = 800 µmol/m²/s
    let rate = photosynthesis_rate(20.0, 0.05, 800.0);
    let temp = temperature_factor(25.0, 25.0);
    println!("\n=== Photosynthesis ===");
    println!("  Rate at PAR=800: {rate:.2} µmol CO₂/m²/s");
    println!("  Temp factor @25°C: {temp:.3}");

    // --- Seasons ---
    println!("\n=== Seasons ===");
    for day in [1, 100, 200, 300] {
        let season = Season::from_day(day);
        println!(
            "  Day {day:>3}: {season:?} — {:.1}h daylight, growth modifier {:.1}",
            season.daylight_hours(),
            season.growth_modifier()
        );
    }

    // --- Root systems ---
    let oak_roots = RootSystem::oak();
    let grass_roots = RootSystem::grass();
    println!("\n=== Root Systems ===");
    println!(
        "  Oak:   {:.1}m deep, stabilization {:.1}",
        oak_roots.max_depth_m,
        oak_roots.stabilization_factor()
    );
    println!(
        "  Grass: {:.1}m deep, stabilization {:.1}",
        grass_roots.max_depth_m,
        grass_roots.stabilization_factor()
    );

    // --- Pollination ---
    let near = pollination_probability(PollinationMethod::Insect, 50.0);
    let far = pollination_probability(PollinationMethod::Insect, 450.0);
    println!("\n=== Pollination (insect) ===");
    println!("  50m:  {near:.2}");
    println!("  450m: {far:.2}");

    // --- Ecosystem ---
    let diversity = shannon_diversity(&[0.4, 0.3, 0.2, 0.1]);
    let npp = net_primary_productivity(1200.0, 500.0);
    let growth = competition_growth(100.0, 0.1, 1000.0, 50.0, 0.5);
    println!("\n=== Ecosystem ===");
    println!("  Shannon diversity (4 species): {diversity:.3}");
    println!("  NPP: {npp:.0} g C/m²/year");
    println!("  Competition growth: {growth:.2}");
}
