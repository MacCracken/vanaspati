use vanaspati::{
    GrowthModel, PollinationMethod, RootSystem, RootType, Season, competition_growth,
    light_compensation_point, net_primary_productivity, photosynthesis_rate,
    pollination_probability, shannon_diversity, temperature_factor, water_use_efficiency,
};

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
    assert!(total > 0.0, "at least some seasons should allow growth");
    assert!(total < 4.0, "not every season is max growth");
}

#[test]
fn photosynthesis_combined_with_temperature() {
    // Photosynthesis at optimum vs cold
    let base_rate = photosynthesis_rate(20.0, 0.05, 800.0);
    let temp_opt = temperature_factor(25.0, 25.0);
    let temp_cold = temperature_factor(5.0, 25.0);

    let effective_opt = base_rate * temp_opt;
    let effective_cold = base_rate * temp_cold;
    assert!(effective_opt > effective_cold);
    assert!(effective_opt > 0.0);
}

#[test]
fn all_re_exports_accessible() {
    // Verify all public API items are accessible from the crate root
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
}

#[test]
fn seasonal_photosynthesis_cycle() {
    // Simulate photosynthesis across seasons using growth modifiers
    let pmax = 20.0; // µmol CO₂/m²/s
    let alpha = 0.05;

    for day in [1_u16, 100, 200, 300] {
        let season = Season::from_day(day);
        let par = season.daylight_hours() * 50.0; // rough PAR from daylight
        let rate = photosynthesis_rate(pmax, alpha, par);
        let modifier = season.growth_modifier();

        if modifier == 0.0 {
            // Winter — growth modifier zeroes out
            assert_eq!(modifier, 0.0);
        } else {
            assert!(
                rate > 0.0,
                "non-winter seasons should produce photosynthesis"
            );
        }
    }
}

#[test]
fn ecosystem_dynamics_consistent() {
    // Equal species should have higher diversity than dominated community
    let equal = shannon_diversity(&[0.25, 0.25, 0.25, 0.25]);
    let dominated = shannon_diversity(&[0.91, 0.03, 0.03, 0.03]);
    assert!(equal > dominated);

    // NPP must be non-negative
    let npp = net_primary_productivity(800.0, 300.0);
    assert_eq!(npp, 500.0);

    // Population below carrying capacity should grow
    let growth = competition_growth(100.0, 0.5, 1000.0, 0.0, 0.0);
    assert!(growth > 0.0);
}
