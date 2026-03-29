/// Lotka-Volterra competition: growth of species in competition.
///
/// `dN/dt = r × N × (1 - (N + α×M) / K)`
///
/// - `population` — N (individuals)
/// - `growth_rate` — r (per time unit)
/// - `carrying_capacity` — K (individuals)
/// - `competitor` — M (competitor population, individuals)
/// - `alpha` — α (competition coefficient, dimensionless)
#[must_use]
pub fn competition_growth(
    population: f32,
    growth_rate: f32,
    carrying_capacity: f32,
    competitor: f32,
    alpha: f32,
) -> f32 {
    if carrying_capacity <= 0.0 {
        tracing::warn!(
            carrying_capacity,
            "competition_growth called with non-positive carrying capacity"
        );
        return 0.0;
    }
    let result =
        growth_rate * population * (1.0 - (population + alpha * competitor) / carrying_capacity);
    tracing::trace!(
        population,
        growth_rate,
        carrying_capacity,
        competitor,
        alpha,
        result,
        "competition_growth"
    );
    result
}

/// Species diversity index (Shannon-Wiener).
///
/// `H = -Σ(p_i × ln(p_i))`
///
/// Caller is responsible for ensuring proportions are in `(0, 1]` and sum to 1.
/// Values ≤ 0 are skipped. No validation is performed.
#[must_use]
pub fn shannon_diversity(proportions: &[f32]) -> f32 {
    let mut h = 0.0_f32;
    for &p in proportions {
        if p > 0.0 {
            h -= p * p.ln();
        }
    }
    tracing::trace!(
        species_count = proportions.len(),
        diversity = h,
        "shannon_diversity"
    );
    h
}

/// Net Primary Productivity (simplified, g C/m²/year).
///
/// `NPP = GPP - R_a`, clamped to zero.
///
/// - `gross_productivity` — GPP (g C/m²/year)
/// - `respiration` — autotrophic respiration (g C/m²/year)
#[must_use]
pub fn net_primary_productivity(gross_productivity: f32, respiration: f32) -> f32 {
    let npp = (gross_productivity - respiration).max(0.0);
    tracing::trace!(
        gross_productivity,
        respiration,
        npp,
        "net_primary_productivity"
    );
    npp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn competition_reduces_growth() {
        let alone = competition_growth(50.0, 0.1, 1000.0, 0.0, 0.5);
        let competing = competition_growth(50.0, 0.1, 1000.0, 200.0, 0.5);
        assert!(competing < alone);
    }

    #[test]
    fn shannon_single_species_zero() {
        let h = shannon_diversity(&[1.0]);
        assert!(h.abs() < 0.001);
    }

    #[test]
    fn shannon_equal_mix_higher() {
        let unequal = shannon_diversity(&[0.9, 0.1]);
        let equal = shannon_diversity(&[0.5, 0.5]);
        assert!(equal > unequal);
    }

    #[test]
    fn npp_positive() {
        assert!(net_primary_productivity(1000.0, 400.0) > 0.0);
    }

    #[test]
    fn npp_zero_when_respiration_exceeds() {
        assert_eq!(net_primary_productivity(100.0, 200.0), 0.0);
    }

    #[test]
    fn competition_zero_carrying_capacity() {
        assert_eq!(competition_growth(50.0, 0.1, 0.0, 0.0, 0.5), 0.0);
    }

    #[test]
    fn competition_at_carrying_capacity() {
        // At K, growth should be ~0
        let g = competition_growth(1000.0, 0.1, 1000.0, 0.0, 0.5);
        assert!(g.abs() < 0.01, "at capacity, growth should be ~0");
    }

    #[test]
    fn shannon_empty_slice() {
        assert_eq!(shannon_diversity(&[]), 0.0);
    }

    #[test]
    fn shannon_diversity_increases_with_species() {
        let two = shannon_diversity(&[0.5, 0.5]);
        let four = shannon_diversity(&[0.25, 0.25, 0.25, 0.25]);
        assert!(four > two, "more equal species = higher diversity");
    }

    #[test]
    fn npp_equal_respiration() {
        assert_eq!(net_primary_productivity(500.0, 500.0), 0.0);
    }
}
