use serde::{Deserialize, Serialize};

/// Seed dispersal mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DispersalMethod {
    Wind,
    Animal,
    Gravity,
    Water,
    Explosive,
}

/// Seed profile for dispersal calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedProfile {
    pub method: DispersalMethod,
    pub mass_g: f32,            // grams
    pub terminal_velocity: f32, // m/s
}

impl SeedProfile {
    /// Dandelion: ultra-light wind dispersal via pappus.
    #[must_use]
    pub fn dandelion() -> Self {
        Self {
            method: DispersalMethod::Wind,
            mass_g: 0.001,
            terminal_velocity: 0.3,
        }
    }

    /// Maple: winged samara, moderate wind dispersal.
    #[must_use]
    pub fn maple() -> Self {
        Self {
            method: DispersalMethod::Wind,
            mass_g: 0.1,
            terminal_velocity: 0.9,
        }
    }

    /// Acorn: heavy, gravity-dispersed (also cached by jays).
    #[must_use]
    pub fn acorn() -> Self {
        Self {
            method: DispersalMethod::Gravity,
            mass_g: 6.0,
            terminal_velocity: 8.0,
        }
    }

    /// Coconut: large, water-dispersed.
    #[must_use]
    pub fn coconut() -> Self {
        Self {
            method: DispersalMethod::Water,
            mass_g: 1400.0,
            terminal_velocity: 12.0,
        }
    }
}

/// Maximum dispersal distance (meters).
///
/// - Wind: `d = (release_height / v_t) × wind_speed`, where `v_t = 4.0 × mass^0.3`
/// - Gravity: `d = release_height × 0.5` (roll factor)
/// - Animal: 5000 m (gut passage distance)
/// - Water: 10000 m (river/ocean transport)
/// - Explosive: `12.0 × (1/mass)^0.5` (lighter seeds go further)
///
/// - `seed_mass_g` — seed mass (grams)
/// - `release_height_m` — height of seed release (meters)
/// - `wind_speed_m_s` — ambient wind speed (meters/second)
#[must_use]
pub fn dispersal_distance(
    method: DispersalMethod,
    seed_mass_g: f32,
    release_height_m: f32,
    wind_speed_m_s: f32,
) -> f32 {
    if seed_mass_g <= 0.0 || release_height_m < 0.0 {
        return 0.0;
    }
    let distance = match method {
        DispersalMethod::Wind => {
            if wind_speed_m_s <= 0.0 {
                return 0.0;
            }
            // Terminal velocity approximation: v_t = 4.0 × mass^0.3 (m/s)
            let v_t = 4.0 * seed_mass_g.powf(0.3);
            if v_t <= 0.0 {
                return 0.0;
            }
            (release_height_m / v_t) * wind_speed_m_s // meters
        }
        DispersalMethod::Gravity => release_height_m * 0.5, // meters, roll factor
        DispersalMethod::Animal => 5000.0,                  // meters, gut passage
        DispersalMethod::Water => 10_000.0,                 // meters, river/ocean
        DispersalMethod::Explosive => {
            12.0 * (1.0 / seed_mass_g).sqrt() // meters, lighter = further
        }
    };
    tracing::trace!(
        ?method,
        seed_mass_g,
        release_height_m,
        wind_speed_m_s,
        distance,
        "dispersal_distance"
    );
    distance
}

/// Probability of a seed landing at a given distance (0.0–1.0).
///
/// Exponential decay kernel: `p = e^(-distance / λ)`
///
/// Characteristic distances (λ): Wind 100m, Animal 500m, Gravity 3m,
/// Water 1000m, Explosive 2m.
///
/// - `distance_m` — distance from parent plant (meters)
#[must_use]
pub fn dispersal_probability(method: DispersalMethod, distance_m: f32) -> f32 {
    if distance_m < 0.0 {
        return 0.0;
    }
    let lambda = match method {
        DispersalMethod::Wind => 100.0,    // meters
        DispersalMethod::Animal => 500.0,  // meters
        DispersalMethod::Gravity => 3.0,   // meters
        DispersalMethod::Water => 1000.0,  // meters
        DispersalMethod::Explosive => 2.0, // meters
    };
    let prob = (-distance_m / lambda).exp();
    tracing::trace!(?method, distance_m, lambda, prob, "dispersal_probability");
    prob
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_dispersal_increases_with_wind_speed() {
        let low = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 2.0);
        let high = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 10.0);
        assert!(high > low);
    }

    #[test]
    fn gravity_dispersal_increases_with_height() {
        let low = dispersal_distance(DispersalMethod::Gravity, 6.0, 5.0, 0.0);
        let high = dispersal_distance(DispersalMethod::Gravity, 6.0, 25.0, 0.0);
        assert!(high > low);
    }

    #[test]
    fn probability_decreases_with_distance() {
        let near = dispersal_probability(DispersalMethod::Wind, 10.0);
        let far = dispersal_probability(DispersalMethod::Wind, 500.0);
        assert!(near > far);
    }

    #[test]
    fn probability_at_zero_is_one() {
        let p = dispersal_probability(DispersalMethod::Wind, 0.0);
        assert!((p - 1.0).abs() < 0.001);
    }

    #[test]
    fn negative_distance_returns_zero() {
        assert_eq!(dispersal_probability(DispersalMethod::Wind, -10.0), 0.0);
    }

    #[test]
    fn zero_mass_returns_zero() {
        assert_eq!(
            dispersal_distance(DispersalMethod::Wind, 0.0, 10.0, 5.0),
            0.0
        );
    }

    #[test]
    fn zero_height_wind_returns_zero() {
        assert_eq!(
            dispersal_distance(DispersalMethod::Wind, 0.1, 0.0, 5.0),
            0.0
        );
    }

    #[test]
    fn zero_wind_returns_zero() {
        assert_eq!(
            dispersal_distance(DispersalMethod::Wind, 0.1, 10.0, 0.0),
            0.0
        );
    }

    #[test]
    fn dandelion_travels_further_than_acorn() {
        let dand = dispersal_distance(DispersalMethod::Wind, 0.001, 10.0, 5.0);
        let acorn = dispersal_distance(DispersalMethod::Gravity, 6.0, 10.0, 5.0);
        assert!(dand > acorn, "dandelion should disperse further than acorn");
    }

    #[test]
    fn explosive_short_range() {
        let d = dispersal_distance(DispersalMethod::Explosive, 0.01, 0.5, 0.0);
        assert!(d < 200.0, "explosive dispersal should be short-range");
        assert!(d > 0.0);
    }

    #[test]
    fn probability_exponential_not_linear() {
        // At half the characteristic distance, probability should be > 0.5 (exponential)
        let p = dispersal_probability(DispersalMethod::Wind, 50.0); // lambda=100
        assert!(
            p > 0.5,
            "exponential decay at half-lambda should be ~0.607, got {p}"
        );
    }

    #[test]
    fn water_dispersal_long_range() {
        let d = dispersal_distance(DispersalMethod::Water, 1400.0, 10.0, 0.0);
        assert_eq!(d, 10_000.0);
    }

    #[test]
    fn preset_masses_positive() {
        assert!(SeedProfile::dandelion().mass_g > 0.0);
        assert!(SeedProfile::maple().mass_g > 0.0);
        assert!(SeedProfile::acorn().mass_g > 0.0);
        assert!(SeedProfile::coconut().mass_g > 0.0);
    }

    #[test]
    fn animal_dispersal_constant() {
        let d1 = dispersal_distance(DispersalMethod::Animal, 0.1, 5.0, 0.0);
        let d2 = dispersal_distance(DispersalMethod::Animal, 10.0, 25.0, 10.0);
        assert_eq!(d1, d2, "animal dispersal is mass/height independent");
    }
}
