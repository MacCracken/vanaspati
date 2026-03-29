use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PollinationMethod {
    Wind,
    Insect,
    Bird,
    Water,
    SelfPollinating,
}

/// Pollination success probability based on distance and method.
///
/// Returns 1.0 for self-pollinating. For other methods, probability
/// decreases linearly from 1.0 at distance 0 to 0.0 at max range.
///
/// - `distance_m` — distance between plants (meters)
///
/// Max ranges: Wind 1000m, Insect 500m, Bird 2000m, Water 100m.
#[must_use]
pub fn pollination_probability(method: PollinationMethod, distance_m: f32) -> f32 {
    if matches!(method, PollinationMethod::SelfPollinating) {
        return 1.0;
    }
    let max_range = match method {
        PollinationMethod::Wind => 1000.0,         // meters
        PollinationMethod::Insect => 500.0,        // meters
        PollinationMethod::Bird => 2000.0,         // meters
        PollinationMethod::Water => 100.0,         // meters
        PollinationMethod::SelfPollinating => 0.0, // handled above
    };
    if distance_m >= max_range || distance_m < 0.0 {
        return 0.0;
    }
    let prob = 1.0 - distance_m / max_range;
    tracing::trace!(
        ?method,
        distance_m,
        max_range,
        prob,
        "pollination_probability"
    );
    prob
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_pollinating_always_succeeds() {
        assert_eq!(
            pollination_probability(PollinationMethod::SelfPollinating, 0.0),
            1.0
        );
    }

    #[test]
    fn probability_decreases_with_distance() {
        let near = pollination_probability(PollinationMethod::Insect, 10.0);
        let far = pollination_probability(PollinationMethod::Insect, 400.0);
        assert!(near > far);
    }

    #[test]
    fn beyond_range_zero() {
        assert_eq!(
            pollination_probability(PollinationMethod::Wind, 2000.0),
            0.0
        );
    }

    #[test]
    fn at_zero_distance_full_probability() {
        assert_eq!(pollination_probability(PollinationMethod::Wind, 0.0), 1.0);
    }

    #[test]
    fn negative_distance_returns_zero() {
        assert_eq!(
            pollination_probability(PollinationMethod::Insect, -10.0),
            0.0
        );
    }

    #[test]
    fn at_exact_max_range_returns_zero() {
        assert_eq!(
            pollination_probability(PollinationMethod::Wind, 1000.0),
            0.0
        );
    }

    #[test]
    fn bird_has_longest_range() {
        let bird = pollination_probability(PollinationMethod::Bird, 1500.0);
        let wind = pollination_probability(PollinationMethod::Wind, 1500.0);
        assert!(bird > 0.0);
        assert_eq!(wind, 0.0);
    }

    #[test]
    fn self_pollinating_ignores_distance() {
        assert_eq!(
            pollination_probability(PollinationMethod::SelfPollinating, 9999.0),
            1.0
        );
    }
}
