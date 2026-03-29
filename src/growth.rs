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

/// Logistic growth model: height(t) = K / (1 + ((K-h0)/h0) × e^(-r×t))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthModel {
    pub max_height: f32,     // K (carrying capacity, meters)
    pub growth_rate: f32,    // r (per day)
    pub initial_height: f32, // h0 (meters)
}

impl GrowthModel {
    /// Height at a given day (meters).
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
}
