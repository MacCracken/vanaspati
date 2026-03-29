use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PollinationMethod { Wind, Insect, Bird, Water, SelfPollinating }

/// Pollination success probability based on distance and method.
#[must_use]
pub fn pollination_probability(method: PollinationMethod, distance_m: f32) -> f32 {
    let max_range = match method {
        PollinationMethod::Wind => 1000.0,
        PollinationMethod::Insect => 500.0,
        PollinationMethod::Bird => 2000.0,
        PollinationMethod::Water => 100.0,
        PollinationMethod::SelfPollinating => 0.0,
    };
    if matches!(method, PollinationMethod::SelfPollinating) { return 1.0; }
    if distance_m > max_range || max_range <= 0.0 { return 0.0; }
    (1.0 - distance_m / max_range).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_pollinating_always_succeeds() {
        assert_eq!(pollination_probability(PollinationMethod::SelfPollinating, 0.0), 1.0);
    }

    #[test]
    fn probability_decreases_with_distance() {
        let near = pollination_probability(PollinationMethod::Insect, 10.0);
        let far = pollination_probability(PollinationMethod::Insect, 400.0);
        assert!(near > far);
    }

    #[test]
    fn beyond_range_zero() {
        assert_eq!(pollination_probability(PollinationMethod::Wind, 2000.0), 0.0);
    }
}
