/// Lotka-Volterra competition: growth of species in competition.
///
/// dN/dt = r × N × (1 - (N + α×M) / K)
///
/// N = population, r = growth rate, K = carrying capacity, M = competitor population, α = competition coefficient.
#[must_use]
pub fn competition_growth(population: f32, growth_rate: f32, carrying_capacity: f32, competitor: f32, alpha: f32) -> f32 {
    if carrying_capacity <= 0.0 { return 0.0; }
    growth_rate * population * (1.0 - (population + alpha * competitor) / carrying_capacity)
}

/// Species diversity index (Shannon-Wiener).
///
/// H = -Σ(p_i × ln(p_i))
#[must_use]
pub fn shannon_diversity(proportions: &[f32]) -> f32 {
    let mut h = 0.0_f32;
    for &p in proportions {
        if p > 0.0 { h -= p * p.ln(); }
    }
    h
}

/// Net Primary Productivity (simplified, g C/m²/year).
#[must_use]
pub fn net_primary_productivity(gross_productivity: f32, respiration: f32) -> f32 {
    (gross_productivity - respiration).max(0.0)
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
}
