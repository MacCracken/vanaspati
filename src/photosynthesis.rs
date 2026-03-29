/// Photosynthesis rate (simplified light response curve).
///
/// `Rate = Pmax × (1 - e^(-α × light / Pmax))`
///
/// - `max_rate` — Pmax (µmol CO₂/m²/s)
/// - `quantum_yield` — α (mol CO₂ / mol photons, typically ~0.05)
/// - `light_intensity` — PAR (µmol photons/m²/s)
#[must_use]
pub fn photosynthesis_rate(max_rate: f32, quantum_yield: f32, light_intensity: f32) -> f32 {
    if max_rate <= 0.0 || light_intensity <= 0.0 {
        return 0.0;
    }
    let rate = max_rate * (1.0 - (-quantum_yield * light_intensity / max_rate).exp());
    tracing::trace!(
        max_rate,
        quantum_yield,
        light_intensity,
        rate,
        "photosynthesis_rate"
    );
    rate
}

/// Light compensation point — PAR level where photosynthesis equals respiration (µmol photons/m²/s).
///
/// - `respiration_rate` — dark respiration (µmol CO₂/m²/s)
/// - `quantum_yield` — α (mol CO₂ / mol photons)
#[must_use]
pub fn light_compensation_point(respiration_rate: f32, quantum_yield: f32) -> f32 {
    if quantum_yield <= 0.0 {
        return 0.0;
    }
    respiration_rate / quantum_yield
}

/// Water use efficiency: carbon gained per water lost (g CO₂ / g H₂O).
#[must_use]
#[inline]
pub fn water_use_efficiency(carbon_fixed: f32, water_transpired: f32) -> f32 {
    if water_transpired <= 0.0 {
        return 0.0;
    }
    carbon_fixed / water_transpired
}

/// Temperature effect on photosynthesis (Gaussian bell curve).
///
/// `factor = e^(-(T - T_opt)² / σ²)`
///
/// σ² = 200 gives a broad curve suitable for C3 plants (optimum ~25°C).
/// Returns 1.0 at optimum, drops toward 0 at extremes.
///
/// - `temp_celsius` — current temperature (°C)
/// - `optimum` — optimal temperature (°C)
#[must_use]
pub fn temperature_factor(temp_celsius: f32, optimum: f32) -> f32 {
    let diff = temp_celsius - optimum;
    let factor = (-diff * diff / 200.0).exp();
    tracing::trace!(temp_celsius, optimum, factor, "temperature_factor");
    factor
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
        assert!(
            (high - 20.0).abs() < 1.0,
            "should approach Pmax at high light"
        );
    }

    #[test]
    fn zero_light_zero_rate() {
        assert_eq!(photosynthesis_rate(20.0, 0.05, 0.0), 0.0);
    }

    #[test]
    fn temp_factor_optimum() {
        let f = temperature_factor(25.0, 25.0);
        assert!(
            (f - 1.0).abs() < 0.01,
            "at optimum temp, factor should be ~1.0"
        );
    }

    #[test]
    fn temp_factor_decreases_away_from_optimum() {
        let opt = temperature_factor(25.0, 25.0);
        let cold = temperature_factor(5.0, 25.0);
        let hot = temperature_factor(45.0, 25.0);
        assert!(cold < opt);
        assert!(hot < opt);
    }

    #[test]
    fn negative_max_rate_returns_zero() {
        assert_eq!(photosynthesis_rate(-5.0, 0.05, 800.0), 0.0);
    }

    #[test]
    fn negative_light_returns_zero() {
        assert_eq!(photosynthesis_rate(20.0, 0.05, -100.0), 0.0);
    }

    #[test]
    fn light_compensation_point_basic() {
        // respiration=2 µmol CO₂/m²/s, α=0.05 → LCP = 40 µmol photons/m²/s
        let lcp = light_compensation_point(2.0, 0.05);
        assert!((lcp - 40.0).abs() < 0.01);
    }

    #[test]
    fn light_compensation_zero_yield() {
        assert_eq!(light_compensation_point(2.0, 0.0), 0.0);
    }

    #[test]
    fn water_use_efficiency_basic() {
        let wue = water_use_efficiency(6.0, 3.0);
        assert!((wue - 2.0).abs() < 0.01);
    }

    #[test]
    fn water_use_efficiency_zero_water() {
        assert_eq!(water_use_efficiency(5.0, 0.0), 0.0);
    }

    #[test]
    fn temp_factor_symmetric() {
        let cold = temperature_factor(15.0, 25.0);
        let hot = temperature_factor(35.0, 25.0);
        assert!(
            (cold - hot).abs() < 0.001,
            "bell curve should be symmetric around optimum"
        );
    }

    #[test]
    fn temp_factor_extreme_cold() {
        let f = temperature_factor(-20.0, 25.0);
        assert!(f < 0.01, "extreme cold should nearly halt photosynthesis");
    }
}
