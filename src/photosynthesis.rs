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
    if max_rate <= 0.0 || quantum_yield <= 0.0 || light_intensity <= 0.0 {
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

// ── Canopy light competition (Beer-Lambert) ───────────────────────────

/// Light intensity at a given depth in a canopy (µmol photons/m²/s).
///
/// Beer-Lambert law: `I(L) = I₀ × e^(-k × L)`
///
/// - `par_above` — PAR above the canopy (µmol photons/m²/s)
/// - `lai_above` — cumulative leaf area index above this point (m² leaf / m² ground)
/// - `extinction_k` — extinction coefficient (dimensionless, typically 0.3–0.8)
///
/// Typical k values: broadleaf 0.5–0.7, conifer 0.3–0.5, grass 0.6–0.8.
#[must_use]
#[inline]
pub fn canopy_light_at_depth(par_above: f32, lai_above: f32, extinction_k: f32) -> f32 {
    if par_above <= 0.0 || lai_above < 0.0 || extinction_k < 0.0 {
        return 0.0;
    }
    let par = par_above * (-extinction_k * lai_above).exp();
    tracing::trace!(
        par_above,
        lai_above,
        extinction_k,
        par,
        "canopy_light_at_depth"
    );
    par
}

/// Fraction of light reaching the forest floor (0.0–1.0).
///
/// `fraction = e^(-k × LAI_total)`
///
/// Useful for determining understory light availability and whether
/// shade-tolerant species can survive beneath a canopy.
///
/// - `total_lai` — total canopy leaf area index (m² leaf / m² ground)
/// - `extinction_k` — extinction coefficient (dimensionless)
#[must_use]
#[inline]
pub fn understory_light_fraction(total_lai: f32, extinction_k: f32) -> f32 {
    if total_lai <= 0.0 || extinction_k < 0.0 {
        return 1.0;
    }
    let fraction = (-extinction_k * total_lai).exp();
    tracing::trace!(
        total_lai,
        extinction_k,
        fraction,
        "understory_light_fraction"
    );
    fraction
}

/// Fraction of light intercepted by the canopy (0.0–1.0).
///
/// `intercepted = 1 - e^(-k × LAI)`
///
/// This is the complement of `understory_light_fraction` — the proportion
/// of incoming light captured by leaves for photosynthesis.
///
/// - `total_lai` — total canopy leaf area index (m² leaf / m² ground)
/// - `extinction_k` — extinction coefficient (dimensionless)
#[must_use]
#[inline]
pub fn light_interception(total_lai: f32, extinction_k: f32) -> f32 {
    if total_lai <= 0.0 || extinction_k < 0.0 {
        return 0.0;
    }
    let intercepted = 1.0 - (-extinction_k * total_lai).exp();
    tracing::trace!(total_lai, extinction_k, intercepted, "light_interception");
    intercepted
}

/// Effective photosynthesis rate under a canopy (µmol CO₂/m²/s).
///
/// Combines Beer-Lambert light extinction with the light response curve:
/// first attenuates PAR through the canopy, then computes photosynthesis
/// at the reduced light level.
///
/// - `max_rate` — Pmax (µmol CO₂/m²/s)
/// - `quantum_yield` — α (mol CO₂ / mol photons)
/// - `par_above_canopy` — PAR above the canopy (µmol photons/m²/s)
/// - `lai_above` — cumulative LAI above this plant (m² leaf / m² ground)
/// - `extinction_k` — extinction coefficient (dimensionless)
#[must_use]
pub fn shaded_photosynthesis_rate(
    max_rate: f32,
    quantum_yield: f32,
    par_above_canopy: f32,
    lai_above: f32,
    extinction_k: f32,
) -> f32 {
    let par_at_depth = canopy_light_at_depth(par_above_canopy, lai_above, extinction_k);
    photosynthesis_rate(max_rate, quantum_yield, par_at_depth)
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

    // --- Canopy light competition tests ---

    #[test]
    fn canopy_light_no_lai() {
        let par = canopy_light_at_depth(1000.0, 0.0, 0.5);
        assert!((par - 1000.0).abs() < 0.1, "no canopy → full light");
    }

    #[test]
    fn canopy_light_decreases_with_lai() {
        let open = canopy_light_at_depth(1000.0, 1.0, 0.5);
        let dense = canopy_light_at_depth(1000.0, 5.0, 0.5);
        assert!(open > dense, "denser canopy → less light");
    }

    #[test]
    fn canopy_light_zero_par() {
        assert_eq!(canopy_light_at_depth(0.0, 3.0, 0.5), 0.0);
    }

    #[test]
    fn canopy_light_negative_par() {
        assert_eq!(canopy_light_at_depth(-100.0, 3.0, 0.5), 0.0);
    }

    #[test]
    fn canopy_light_known_value() {
        // I = 1000 × e^(-0.5 × 3) = 1000 × e^(-1.5) ≈ 223.1
        let par = canopy_light_at_depth(1000.0, 3.0, 0.5);
        assert!((par - 223.1).abs() < 1.0, "got {par}");
    }

    #[test]
    fn understory_no_canopy() {
        assert!((understory_light_fraction(0.0, 0.5) - 1.0).abs() < 0.01);
    }

    #[test]
    fn understory_dense_canopy() {
        let f = understory_light_fraction(6.0, 0.5);
        assert!(f < 0.1, "LAI=6 should block most light, got {f}");
    }

    #[test]
    fn understory_plus_interception_equals_one() {
        let lai = 4.0;
        let k = 0.6;
        let under = understory_light_fraction(lai, k);
        let inter = light_interception(lai, k);
        assert!(
            (under + inter - 1.0).abs() < 0.001,
            "understory + interception should = 1.0"
        );
    }

    #[test]
    fn light_interception_zero_lai() {
        assert_eq!(light_interception(0.0, 0.5), 0.0);
    }

    #[test]
    fn light_interception_increases_with_lai() {
        let sparse = light_interception(1.0, 0.5);
        let dense = light_interception(6.0, 0.5);
        assert!(dense > sparse);
    }

    #[test]
    fn shaded_photosynthesis_less_than_open() {
        let open = photosynthesis_rate(20.0, 0.05, 1000.0);
        let shaded = shaded_photosynthesis_rate(20.0, 0.05, 1000.0, 3.0, 0.5);
        assert!(shaded < open, "shaded plant should photosynthesize less");
        assert!(shaded > 0.0, "but still some");
    }

    #[test]
    fn shaded_photosynthesis_no_canopy_equals_open() {
        let open = photosynthesis_rate(20.0, 0.05, 1000.0);
        let no_shade = shaded_photosynthesis_rate(20.0, 0.05, 1000.0, 0.0, 0.5);
        assert!((open - no_shade).abs() < 0.01);
    }

    #[test]
    fn shaded_photosynthesis_very_dense_near_zero() {
        let deep = shaded_photosynthesis_rate(20.0, 0.05, 1000.0, 10.0, 0.7);
        assert!(deep < 1.0, "LAI=10, k=0.7 should nearly extinguish light");
    }

    // --- Negative input guard tests ---

    #[test]
    fn negative_quantum_yield_returns_zero() {
        assert_eq!(photosynthesis_rate(20.0, -0.05, 800.0), 0.0);
    }

    #[test]
    fn zero_quantum_yield_returns_zero() {
        assert_eq!(photosynthesis_rate(20.0, 0.0, 800.0), 0.0);
    }

    #[test]
    fn negative_extinction_k_canopy_returns_zero() {
        assert_eq!(canopy_light_at_depth(1000.0, 3.0, -0.5), 0.0);
    }

    #[test]
    fn negative_extinction_k_understory_returns_one() {
        assert_eq!(understory_light_fraction(3.0, -0.5), 1.0);
    }

    #[test]
    fn negative_extinction_k_interception_returns_zero() {
        assert_eq!(light_interception(3.0, -0.5), 0.0);
    }
}
