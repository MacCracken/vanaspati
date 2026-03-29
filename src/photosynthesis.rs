/// Photosynthesis rate (simplified light response curve).
///
/// Rate = Pmax × (1 - e^(-α × light / Pmax))
///
/// Pmax = maximum rate (µmol CO₂/m²/s), α = quantum yield, light in µmol photons/m²/s.
#[must_use]
pub fn photosynthesis_rate(max_rate: f32, quantum_yield: f32, light_intensity: f32) -> f32 {
    if max_rate <= 0.0 || light_intensity <= 0.0 { return 0.0; }
    max_rate * (1.0 - (-quantum_yield * light_intensity / max_rate).exp())
}

/// Light compensation point — light level where photosynthesis equals respiration.
#[must_use]
pub fn light_compensation_point(respiration_rate: f32, quantum_yield: f32) -> f32 {
    if quantum_yield <= 0.0 { return 0.0; }
    respiration_rate / quantum_yield
}

/// Water use efficiency: carbon gained per water lost (g CO₂ / g H₂O).
#[must_use]
#[inline]
pub fn water_use_efficiency(carbon_fixed: f32, water_transpired: f32) -> f32 {
    if water_transpired <= 0.0 { return 0.0; }
    carbon_fixed / water_transpired
}

/// Temperature effect on photosynthesis (bell curve, optimum at ~25°C for C3 plants).
#[must_use]
pub fn temperature_factor(temp_celsius: f32, optimum: f32) -> f32 {
    let diff = temp_celsius - optimum;
    (-diff * diff / 200.0).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_increases_with_light() {
        let low = photosynthesis_rate(20.0, 0.05, 100.0);
        let high = photosynthesis_rate(20.0, 0.05, 1000.0);
        assert!(high > low);
    }

    #[test]
    fn rate_saturates() {
        let high = photosynthesis_rate(20.0, 0.05, 2000.0);
        assert!((high - 20.0).abs() < 1.0, "should approach Pmax at high light");
    }

    #[test]
    fn zero_light_zero_rate() {
        assert_eq!(photosynthesis_rate(20.0, 0.05, 0.0), 0.0);
    }

    #[test]
    fn temp_factor_optimum() {
        let f = temperature_factor(25.0, 25.0);
        assert!((f - 1.0).abs() < 0.01, "at optimum temp, factor should be ~1.0");
    }

    #[test]
    fn temp_factor_decreases_away_from_optimum() {
        let opt = temperature_factor(25.0, 25.0);
        let cold = temperature_factor(5.0, 25.0);
        let hot = temperature_factor(45.0, 25.0);
        assert!(cold < opt);
        assert!(hot < opt);
    }
}
