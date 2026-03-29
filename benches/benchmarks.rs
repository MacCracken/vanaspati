use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vanaspati::{
    GrowthModel, PollinationMethod, RootSystem, Season, competition_growth,
    net_primary_productivity, photosynthesis_rate, pollination_probability, shannon_diversity,
    temperature_factor,
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
}

fn bench_photosynthesis(c: &mut Criterion) {
    c.bench_function("photosynthesis_rate", |b| {
        b.iter(|| photosynthesis_rate(black_box(20.0), black_box(0.05), black_box(800.0)))
    });
    c.bench_function("temperature_factor", |b| {
        b.iter(|| temperature_factor(black_box(25.0), black_box(25.0)))
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

criterion_group!(
    benches,
    bench_growth,
    bench_photosynthesis,
    bench_season,
    bench_root,
    bench_pollination,
    bench_ecosystem,
);
criterion_main!(benches);
