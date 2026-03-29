use serde::{Deserialize, Serialize};

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

/// Temperature effect on photosynthesis (Gaussian bell curve, C3 default).
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

/// Photosynthesis pathway type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PhotosynthesisPathway {
    /// C3 — most plants (trees, cool-season grasses, wheat, rice).
    C3,
    /// C4 — tropical/warm-season grasses (corn, sugarcane, sorghum).
    C4,
    /// CAM — succulents, cacti, pineapple.
    CAM,
}

/// Pathway-specific parameters: (optimum_temp °C, quantum_yield, max_rate µmol CO₂/m²/s).
///
/// - C3: (25.0, 0.05, 20.0) — broad temperature range, moderate efficiency
/// - C4: (32.0, 0.06, 40.0) — warm-adapted, CO₂ concentrating, high Pmax
/// - CAM: (28.0, 0.04, 10.0) — desert-adapted, temporal CO₂ separation, low Pmax
#[must_use]
pub fn pathway_params(pathway: PhotosynthesisPathway) -> (f32, f32, f32) {
    let params = match pathway {
        PhotosynthesisPathway::C3 => (25.0, 0.05, 20.0),
        PhotosynthesisPathway::C4 => (32.0, 0.06, 40.0),
        PhotosynthesisPathway::CAM => (28.0, 0.04, 10.0),
    };
    tracing::trace!(
        ?pathway,
        optimum = params.0,
        quantum_yield = params.1,
        max_rate = params.2,
        "pathway_params"
    );
    params
}

/// Temperature effect for C4 photosynthesis (narrow Gaussian, optimum 32°C).
///
/// `factor = e^(-(T - 32)² / 150)`
///
/// σ² = 150 (narrower than C3) — C4 plants are warm-climate specialists.
///
/// - `temp_celsius` — current temperature (°C)
#[must_use]
pub fn temperature_factor_c4(temp_celsius: f32) -> f32 {
    let diff = temp_celsius - 32.0;
    let factor = (-diff * diff / 150.0).exp();
    tracing::trace!(temp_celsius, factor, "temperature_factor_c4");
    factor
}

/// Temperature effect for CAM photosynthesis (broad Gaussian, optimum 28°C).
///
/// `factor = e^(-(T - 28)² / 250)`
///
/// σ² = 250 (broader than C3) — CAM plants tolerate large diurnal temperature swings.
///
/// - `temp_celsius` — current temperature (°C)
#[must_use]
pub fn temperature_factor_cam(temp_celsius: f32) -> f32 {
    let diff = temp_celsius - 28.0;
    let factor = (-diff * diff / 250.0).exp();
    tracing::trace!(temp_celsius, factor, "temperature_factor_cam");
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

    // --- C4/CAM tests ---

    #[test]
    fn c4_optimum_at_32() {
        let f = temperature_factor_c4(32.0);
        assert!((f - 1.0).abs() < 0.01);
    }

    #[test]
    fn cam_optimum_at_28() {
        let f = temperature_factor_cam(28.0);
        assert!((f - 1.0).abs() < 0.01);
    }

    #[test]
    fn c4_narrower_than_c3() {
        // At 15°C (far from both optima), C4 should drop more than C3
        let c3 = temperature_factor(15.0, 25.0); // 10°C from optimum
        let c4 = temperature_factor_c4(15.0); // 17°C from optimum + narrower σ²
        assert!(c4 < c3, "C4 should be more sensitive to cold");
    }

    #[test]
    fn cam_broader_than_c3() {
        // At equal distance from their respective optima,
        // CAM (σ²=250) drops less than C3 (σ²=200)
        let c3 = temperature_factor(10.0, 25.0); // 15°C from optimum
        let cam = temperature_factor_cam(13.0); // 15°C from optimum
        assert!(
            cam > c3,
            "CAM should be more tolerant of temperature deviation"
        );
    }

    #[test]
    fn pathway_params_c3() {
        let (t, a, p) = pathway_params(PhotosynthesisPathway::C3);
        assert_eq!(t, 25.0);
        assert_eq!(a, 0.05);
        assert_eq!(p, 20.0);
    }

    #[test]
    fn pathway_params_c4() {
        let (t, a, p) = pathway_params(PhotosynthesisPathway::C4);
        assert_eq!(t, 32.0);
        assert_eq!(a, 0.06);
        assert_eq!(p, 40.0);
    }

    #[test]
    fn pathway_params_cam() {
        let (t, a, p) = pathway_params(PhotosynthesisPathway::CAM);
        assert_eq!(t, 28.0);
        assert_eq!(a, 0.04);
        assert_eq!(p, 10.0);
    }

    #[test]
    fn c4_higher_max_rate() {
        let (_, _, c3_pmax) = pathway_params(PhotosynthesisPathway::C3);
        let (_, _, c4_pmax) = pathway_params(PhotosynthesisPathway::C4);
        assert!(c4_pmax > c3_pmax);
    }

    #[test]
    fn cam_lower_quantum_yield() {
        let (_, c3_a, _) = pathway_params(PhotosynthesisPathway::C3);
        let (_, cam_a, _) = pathway_params(PhotosynthesisPathway::CAM);
        assert!(cam_a < c3_a);
    }

    #[test]
    fn c4_extreme_cold_near_zero() {
        let f = temperature_factor_c4(-10.0);
        assert!(f < 0.01);
    }
}
