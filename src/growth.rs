use serde::{Deserialize, Serialize};

/// Plant growth stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GrowthStage {
    Seed,
    Germination,
    Seedling,
    Vegetative,
    Flowering,
    Fruiting,
    Senescence,
    Dormant,
}

/// Growth stage from height relative to max height.
///
/// - < 1% → Seed
/// - 1–5% → Germination
/// - 5–15% → Seedling
/// - 15–60% → Vegetative
/// - 60–80% → Flowering
/// - 80–95% → Fruiting
/// - > 95% → Senescence
///
/// - `current_height` — current plant height (meters)
/// - `max_height` — species maximum height (meters)
#[must_use]
pub fn growth_stage(current_height: f32, max_height: f32) -> GrowthStage {
    if max_height <= 0.0 || current_height <= 0.0 {
        return GrowthStage::Seed;
    }
    let fraction = (current_height / max_height).clamp(0.0, 1.0);
    let stage = match fraction {
        f if f < 0.01 => GrowthStage::Seed,
        f if f < 0.05 => GrowthStage::Germination,
        f if f < 0.15 => GrowthStage::Seedling,
        f if f < 0.60 => GrowthStage::Vegetative,
        f if f < 0.80 => GrowthStage::Flowering,
        f if f < 0.95 => GrowthStage::Fruiting,
        _ => GrowthStage::Senescence,
    };
    tracing::trace!(current_height, max_height, fraction, ?stage, "growth_stage");
    stage
}

/// Logistic growth model: height(t) = K / (1 + ((K-h0)/h0) × e^(-r×t))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthModel {
    pub max_height: f32,     // K (carrying capacity, meters)
    pub growth_rate: f32,    // r (per day)
    pub initial_height: f32, // h0 (meters)
}

impl GrowthModel {
    /// Height at a given day (meters).
    ///
    /// Works best when `initial_height` < `max_height`. If `initial_height` >= `max_height`,
    /// the curve converges toward `max_height` from above.
    #[must_use]
    pub fn height_at_day(&self, day: f32) -> f32 {
        if self.initial_height <= 0.0 {
            return 0.0;
        }
        let k = self.max_height;
        let h0 = self.initial_height;
        let r = self.growth_rate;
        let height = k / (1.0 + ((k - h0) / h0) * (-r * day).exp());
        tracing::trace!(day, height, max_height = k, "height_at_day");
        height
    }

    /// Growth rate at current height (derivative of logistic, meters/day).
    ///
    /// Returns negative if `current_height` exceeds `max_height` (overshoot correction).
    #[must_use]
    pub fn daily_growth(&self, current_height: f32) -> f32 {
        let k = self.max_height;
        if k <= 0.0 {
            return 0.0;
        }
        let rate = self.growth_rate * current_height * (1.0 - current_height / k);
        tracing::trace!(current_height, rate, "daily_growth");
        rate
    }

    /// Oak tree: slow growth, tall.
    #[must_use]
    pub fn oak() -> Self {
        Self {
            max_height: 25.0,
            growth_rate: 0.005,
            initial_height: 0.1,
        }
    }
    /// Bamboo: fast growth.
    #[must_use]
    pub fn bamboo() -> Self {
        Self {
            max_height: 20.0,
            growth_rate: 0.05,
            initial_height: 0.1,
        }
    }
    /// Grass: very fast, short.
    #[must_use]
    pub fn grass() -> Self {
        Self {
            max_height: 0.5,
            growth_rate: 0.1,
            initial_height: 0.01,
        }
    }
}

/// Water stress factor on growth (0.0–1.0).
///
/// Growth is more sensitive to drought than photosynthesis — plants reduce
/// growth allocation before photosynthetic capacity declines. Full growth
/// above 60% relative water content, linear decline to zero at wilting.
///
/// `factor = min(1.0, RWC / 0.6)`
///
/// Based on Hsiao (1973) — cell expansion is the first process inhibited
/// by water deficit, well before photosynthesis declines.
///
/// - `relative_water_content` — fraction of plant-available water (0.0–1.0)
#[must_use]
#[inline]
pub fn water_stress_growth_factor(relative_water_content: f32) -> f32 {
    if relative_water_content <= 0.0 {
        return 0.0;
    }
    let factor = (relative_water_content / 0.6).clamp(0.0, 1.0);
    tracing::trace!(relative_water_content, factor, "water_stress_growth_factor");
    factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logistic_starts_at_initial() {
        let m = GrowthModel::oak();
        assert!((m.height_at_day(0.0) - 0.1).abs() < 0.01);
    }

    #[test]
    fn logistic_approaches_max() {
        let m = GrowthModel::grass();
        let h = m.height_at_day(365.0);
        assert!(
            (h - m.max_height).abs() < 0.01,
            "should approach max height, got {h}"
        );
    }

    #[test]
    fn bamboo_grows_faster_than_oak() {
        let oak = GrowthModel::oak().height_at_day(100.0);
        let bamboo = GrowthModel::bamboo().height_at_day(100.0);
        assert!(bamboo > oak, "bamboo should be taller at 100 days");
    }

    #[test]
    fn daily_growth_zero_at_max() {
        let m = GrowthModel::oak();
        let g = m.daily_growth(m.max_height);
        assert!(g.abs() < 0.001, "growth should be ~0 at max height");
    }

    #[test]
    fn daily_growth_positive_mid() {
        let m = GrowthModel::oak();
        let g = m.daily_growth(5.0);
        assert!(g > 0.0);
    }

    #[test]
    fn zero_initial_height_returns_zero() {
        let m = GrowthModel {
            max_height: 10.0,
            growth_rate: 0.01,
            initial_height: 0.0,
        };
        assert_eq!(m.height_at_day(100.0), 0.0);
    }

    #[test]
    fn zero_max_height_no_daily_growth() {
        let m = GrowthModel {
            max_height: 0.0,
            growth_rate: 0.01,
            initial_height: 0.1,
        };
        assert_eq!(m.daily_growth(5.0), 0.0);
    }

    #[test]
    fn grass_reaches_max_fast() {
        let g = GrowthModel::grass();
        let h = g.height_at_day(100.0);
        assert!(
            (h - g.max_height).abs() < 0.01,
            "grass should reach max well before a year"
        );
    }

    #[test]
    fn growth_monotonically_increases() {
        let oak = GrowthModel::oak();
        let mut prev = 0.0_f32;
        for day in (0..=365).step_by(10) {
            let h = oak.height_at_day(day as f32);
            assert!(h >= prev, "height should never decrease: day {day}");
            prev = h;
        }
    }

    // --- growth_stage tests ---

    #[test]
    fn stage_seed_at_zero() {
        assert_eq!(growth_stage(0.0, 25.0), GrowthStage::Seed);
    }

    #[test]
    fn stage_seed_negative_height() {
        assert_eq!(growth_stage(-1.0, 25.0), GrowthStage::Seed);
    }

    #[test]
    fn stage_seed_zero_max() {
        assert_eq!(growth_stage(5.0, 0.0), GrowthStage::Seed);
    }

    #[test]
    fn stage_progression() {
        let max = 100.0;
        assert_eq!(growth_stage(0.5, max), GrowthStage::Seed); // 0.5%
        assert_eq!(growth_stage(2.0, max), GrowthStage::Germination); // 2%
        assert_eq!(growth_stage(10.0, max), GrowthStage::Seedling); // 10%
        assert_eq!(growth_stage(40.0, max), GrowthStage::Vegetative); // 40%
        assert_eq!(growth_stage(70.0, max), GrowthStage::Flowering); // 70%
        assert_eq!(growth_stage(90.0, max), GrowthStage::Fruiting); // 90%
        assert_eq!(growth_stage(98.0, max), GrowthStage::Senescence); // 98%
    }

    #[test]
    fn stage_clamped_above_max() {
        // height > max_height still returns Senescence (clamped to 1.0)
        assert_eq!(growth_stage(30.0, 25.0), GrowthStage::Senescence);
    }

    // --- water_stress_growth_factor tests ---

    #[test]
    fn growth_water_stress_full() {
        assert_eq!(water_stress_growth_factor(1.0), 1.0);
    }

    #[test]
    fn growth_water_stress_above_threshold() {
        assert_eq!(water_stress_growth_factor(0.8), 1.0);
    }

    #[test]
    fn growth_water_stress_at_threshold() {
        assert!((water_stress_growth_factor(0.6) - 1.0).abs() < 0.01);
    }

    #[test]
    fn growth_water_stress_half_threshold() {
        assert!((water_stress_growth_factor(0.3) - 0.5).abs() < 0.01);
    }

    #[test]
    fn growth_water_stress_wilted() {
        assert_eq!(water_stress_growth_factor(0.0), 0.0);
    }

    #[test]
    fn growth_water_stress_negative() {
        assert_eq!(water_stress_growth_factor(-0.1), 0.0);
    }

    #[test]
    fn growth_more_sensitive_than_photosynthesis() {
        // At RWC=0.5, growth should be reduced but photosynthesis should not
        let growth_f = water_stress_growth_factor(0.5);
        assert!(growth_f < 1.0, "growth should be stressed at RWC=0.5");
        // Photosynthesis threshold is 0.4, so at 0.5 it's still 1.0
        let photo_f = crate::photosynthesis::water_stress_factor(0.5);
        assert!(
            growth_f < photo_f,
            "growth should be more sensitive: growth={growth_f}, photo={photo_f}"
        );
    }

    #[test]
    fn stage_integrates_with_growth_model() {
        let oak = GrowthModel::oak();
        let h0 = oak.height_at_day(0.0);
        let h_mid = oak.height_at_day(500.0);
        let h_late = oak.height_at_day(5000.0);
        assert_eq!(growth_stage(h0, oak.max_height), GrowthStage::Seed);
        assert_ne!(growth_stage(h_mid, oak.max_height), GrowthStage::Seed);
        assert!(matches!(
            growth_stage(h_late, oak.max_height),
            GrowthStage::Fruiting | GrowthStage::Senescence
        ));
    }
}
