//! Fire ecology — fire adaptation traits, bark protection,
//! post-fire regeneration, and serotiny.
//!
//! Fire is a major ecological force shaping plant communities worldwide.
//! Species have evolved diverse strategies: thick bark for survival,
//! resprouting from belowground organs, or serotinous cones that
//! release seeds only after fire.

use serde::{Deserialize, Serialize};

/// Fire adaptation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FireStrategy {
    /// Fire-sensitive — no adaptations, killed by moderate fire.
    /// Examples: most tropical rainforest species, beech.
    Sensitive,
    /// Resprouter — survives fire via belowground buds/lignotuber.
    /// Examples: eucalyptus (many species), oaks, grasses.
    Resprouter,
    /// Thick-barked — insulates cambium from heat damage.
    /// Examples: ponderosa pine, longleaf pine, cork oak.
    ThickBarked,
    /// Serotinous — retains seeds in sealed cones/fruits until fire.
    /// Examples: jack pine, lodgepole pine, banksia.
    Serotinous,
}

/// Bark protection factor (0.0–1.0) for a fire strategy.
///
/// How well the bark insulates the cambium from fire heat.
/// 0.0 = no protection, 1.0 = complete protection.
///
/// - `strategy` — fire adaptation strategy
#[must_use]
#[inline]
pub fn bark_protection(strategy: FireStrategy) -> f32 {
    let protection = match strategy {
        FireStrategy::Sensitive => 0.1,   // thin bark
        FireStrategy::Resprouter => 0.3,  // moderate — survives via resprouting, not bark
        FireStrategy::ThickBarked => 0.8, // thick insulating bark
        FireStrategy::Serotinous => 0.2,  // thin bark — species relies on seed release
    };
    tracing::trace!(?strategy, protection, "bark_protection");
    protection
}

/// Post-fire resprouting vigor (0.0–1.0).
///
/// How quickly and vigorously the plant regrows after fire.
/// Based on belowground carbohydrate reserves and bud bank.
///
/// Returns a multiplier on base growth rate for the first year post-fire.
///
/// - `strategy` — fire adaptation strategy
/// - `fire_intensity` — fire intensity (0.0–1.0)
#[must_use]
pub fn resprout_vigor(strategy: FireStrategy, fire_intensity: f32) -> f32 {
    let intensity = fire_intensity.clamp(0.0, 1.0);
    let base_vigor = match strategy {
        FireStrategy::Sensitive => 0.0,   // cannot resprout
        FireStrategy::Resprouter => 0.9,  // strong resprouting
        FireStrategy::ThickBarked => 0.4, // some epicormic resprouting
        FireStrategy::Serotinous => 0.1,  // minimal — relies on seeds instead
    };
    // Very intense fires can damage belowground reserves
    let vigor = base_vigor * (1.0 - 0.3 * intensity);
    tracing::trace!(
        ?strategy,
        fire_intensity,
        base_vigor,
        vigor,
        "resprout_vigor"
    );
    vigor.max(0.0)
}

/// Serotinous seed release (seeds released per fire event).
///
/// Serotinous species store seeds for years, releasing them en masse after fire.
/// Non-serotinous species get 0.
///
/// `released = seed_bank × (0.5 + 0.5 × fire_intensity)`
///
/// Hotter fires open more cones/fruits.
///
/// - `strategy` — fire adaptation strategy
/// - `seed_bank` — accumulated seeds (count)
/// - `fire_intensity` — fire intensity (0.0–1.0)
#[must_use]
pub fn serotinous_release(strategy: FireStrategy, seed_bank: f32, fire_intensity: f32) -> f32 {
    if strategy != FireStrategy::Serotinous || seed_bank <= 0.0 {
        return 0.0;
    }
    let intensity = fire_intensity.clamp(0.0, 1.0);
    let released = seed_bank * (0.5 + 0.5 * intensity);
    tracing::trace!(seed_bank, fire_intensity, released, "serotinous_release");
    released
}

/// Post-fire establishment advantage (multiplier on establishment probability).
///
/// After fire clears canopy and litter, many species benefit from:
/// - Increased light (canopy removed)
/// - Nutrient flush (ash fertilization)
/// - Reduced competition
///
/// Pioneer species benefit most. Climax species are disadvantaged.
///
/// `advantage = base × (1.0 + 0.5 × fire_intensity)`
///
/// - `strategy` — fire adaptation strategy
/// - `fire_intensity` — fire intensity (0.0–1.0)
#[must_use]
pub fn post_fire_establishment(strategy: FireStrategy, fire_intensity: f32) -> f32 {
    let intensity = fire_intensity.clamp(0.0, 1.0);
    let base = match strategy {
        FireStrategy::Sensitive => 0.5, // disadvantaged — competitors removed but also self
        FireStrategy::Resprouter => 1.5, // good — quick regrowth
        FireStrategy::ThickBarked => 1.2, // survived → less competition
        FireStrategy::Serotinous => 2.0, // best — massive seed release into open ground
    };
    let advantage = base * (1.0 + 0.5 * intensity);
    tracing::trace!(
        ?strategy,
        fire_intensity,
        base,
        advantage,
        "post_fire_establishment"
    );
    advantage
}

/// Fire return interval — typical years between fires for different biomes.
///
/// Returns expected fire interval (years). Useful for calculating
/// annual fire probability as `1.0 / interval`.
///
/// - `is_fire_prone` — true for fire-prone biomes (savanna, chaparral, boreal)
#[must_use]
#[inline]
pub fn fire_return_interval_years(is_fire_prone: bool) -> f32 {
    if is_fire_prone {
        15.0 // savanna/chaparral: 5–25 year cycles
    } else {
        200.0 // mesic forests: rare fire
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- bark protection ---

    #[test]
    fn thick_bark_most_protected() {
        assert!(
            bark_protection(FireStrategy::ThickBarked) > bark_protection(FireStrategy::Sensitive)
        );
        assert!(
            bark_protection(FireStrategy::ThickBarked) > bark_protection(FireStrategy::Resprouter)
        );
        assert!(
            bark_protection(FireStrategy::ThickBarked) > bark_protection(FireStrategy::Serotinous)
        );
    }

    #[test]
    fn protection_in_range() {
        for s in [
            FireStrategy::Sensitive,
            FireStrategy::Resprouter,
            FireStrategy::ThickBarked,
            FireStrategy::Serotinous,
        ] {
            let p = bark_protection(s);
            assert!((0.0..=1.0).contains(&p), "{s:?}: {p}");
        }
    }

    // --- resprout vigor ---

    #[test]
    fn sensitive_cannot_resprout() {
        assert_eq!(resprout_vigor(FireStrategy::Sensitive, 0.5), 0.0);
    }

    #[test]
    fn resprouter_high_vigor() {
        let v = resprout_vigor(FireStrategy::Resprouter, 0.3);
        assert!(v > 0.5, "resprouter should have high vigor, got {v}");
    }

    #[test]
    fn intense_fire_reduces_vigor() {
        let low = resprout_vigor(FireStrategy::Resprouter, 0.2);
        let high = resprout_vigor(FireStrategy::Resprouter, 0.9);
        assert!(low > high, "intense fire should reduce resprout vigor");
    }

    #[test]
    fn vigor_never_negative() {
        let v = resprout_vigor(FireStrategy::Serotinous, 1.0);
        assert!(v >= 0.0);
    }

    // --- serotinous release ---

    #[test]
    fn only_serotinous_releases() {
        assert_eq!(
            serotinous_release(FireStrategy::Resprouter, 1000.0, 1.0),
            0.0
        );
        assert_eq!(
            serotinous_release(FireStrategy::ThickBarked, 1000.0, 1.0),
            0.0
        );
    }

    #[test]
    fn serotinous_releases_seeds() {
        let released = serotinous_release(FireStrategy::Serotinous, 1000.0, 0.8);
        assert!(released > 500.0, "should release majority of seed bank");
    }

    #[test]
    fn hotter_fire_more_seeds() {
        let cool = serotinous_release(FireStrategy::Serotinous, 1000.0, 0.2);
        let hot = serotinous_release(FireStrategy::Serotinous, 1000.0, 0.9);
        assert!(hot > cool);
    }

    #[test]
    fn serotinous_zero_bank() {
        assert_eq!(serotinous_release(FireStrategy::Serotinous, 0.0, 1.0), 0.0);
    }

    // --- post-fire establishment ---

    #[test]
    fn serotinous_best_post_fire() {
        let sero = post_fire_establishment(FireStrategy::Serotinous, 0.5);
        let sens = post_fire_establishment(FireStrategy::Sensitive, 0.5);
        assert!(sero > sens);
    }

    #[test]
    fn establishment_increases_with_intensity() {
        let low = post_fire_establishment(FireStrategy::Resprouter, 0.2);
        let high = post_fire_establishment(FireStrategy::Resprouter, 0.8);
        assert!(high > low);
    }

    #[test]
    fn all_establishments_positive() {
        for s in [
            FireStrategy::Sensitive,
            FireStrategy::Resprouter,
            FireStrategy::ThickBarked,
            FireStrategy::Serotinous,
        ] {
            assert!(post_fire_establishment(s, 0.5) > 0.0, "{s:?}");
        }
    }

    // --- fire return interval ---

    #[test]
    fn fire_prone_shorter_interval() {
        assert!(fire_return_interval_years(true) < fire_return_interval_years(false));
    }

    #[test]
    fn intervals_positive() {
        assert!(fire_return_interval_years(true) > 0.0);
        assert!(fire_return_interval_years(false) > 0.0);
    }
}
