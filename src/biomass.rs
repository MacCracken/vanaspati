use serde::{Deserialize, Serialize};

/// Carbon pool partitioned across plant organs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomassPool {
    pub leaf_kg: f32,         // kilograms
    pub stem_kg: f32,         // kilograms
    pub root_kg: f32,         // kilograms
    pub reproductive_kg: f32, // kilograms
}

impl BiomassPool {
    /// Total biomass across all pools (kg).
    #[must_use]
    pub fn total_kg(&self) -> f32 {
        self.leaf_kg + self.stem_kg + self.root_kg + self.reproductive_kg
    }

    /// Oak: large tree, stem-heavy.
    #[must_use]
    pub fn oak() -> Self {
        Self {
            leaf_kg: 50.0,
            stem_kg: 2000.0,
            root_kg: 500.0,
            reproductive_kg: 20.0,
        }
    }

    /// Bamboo: fast-growing, stem-dominant.
    #[must_use]
    pub fn bamboo() -> Self {
        Self {
            leaf_kg: 15.0,
            stem_kg: 100.0,
            root_kg: 30.0,
            reproductive_kg: 5.0,
        }
    }

    /// Grass: leaf-dominant, shallow roots.
    #[must_use]
    pub fn grass() -> Self {
        Self {
            leaf_kg: 0.02,
            stem_kg: 0.01,
            root_kg: 0.015,
            reproductive_kg: 0.005,
        }
    }
}

/// Carbon allocation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AllocationStrategy {
    /// Balanced vegetative growth (leaf 0.30, stem 0.35, root 0.30, repro 0.05).
    Balanced,
    /// Drought/nutrient stress — heavy root investment (root 0.65).
    StressedRoot,
    /// Flowering/fruiting phase — heavy reproductive investment (repro 0.60).
    Reproductive,
}

/// Partition carbon gain across plant organs.
///
/// Fractions per strategy:
/// - Balanced: leaf 0.30, stem 0.35, root 0.30, repro 0.05
/// - StressedRoot: leaf 0.15, stem 0.15, root 0.65, repro 0.05
/// - Reproductive: leaf 0.15, stem 0.15, root 0.10, repro 0.60
///
/// - `total_carbon_g` — total carbon gain to allocate (grams)
/// - `strategy` — allocation strategy
///
/// Returns pool in kg (input grams / 1000).
#[must_use]
pub fn allocate(total_carbon_g: f32, strategy: AllocationStrategy) -> BiomassPool {
    if total_carbon_g <= 0.0 {
        return BiomassPool {
            leaf_kg: 0.0,
            stem_kg: 0.0,
            root_kg: 0.0,
            reproductive_kg: 0.0,
        };
    }
    let kg = total_carbon_g / 1000.0;
    let (leaf, stem, root, repro) = match strategy {
        AllocationStrategy::Balanced => (0.30, 0.35, 0.30, 0.05),
        AllocationStrategy::StressedRoot => (0.15, 0.15, 0.65, 0.05),
        AllocationStrategy::Reproductive => (0.15, 0.15, 0.10, 0.60),
    };
    let pool = BiomassPool {
        leaf_kg: kg * leaf,
        stem_kg: kg * stem,
        root_kg: kg * root,
        reproductive_kg: kg * repro,
    };
    tracing::trace!(
        total_carbon_g,
        ?strategy,
        total_kg = pool.total_kg(),
        "allocate"
    );
    pool
}

/// Stem diameter from height using allometric scaling (meters).
///
/// `diameter = coefficient × height^0.67`
///
/// Based on metabolic scaling theory (West, Brown, Enquist 1999).
/// Typical coefficients: oak 0.04, bamboo 0.008, grass 0.002.
///
/// - `height_m` — plant height (meters)
/// - `species_coefficient` — dimensionless scaling coefficient
#[must_use]
pub fn height_to_diameter(height_m: f32, species_coefficient: f32) -> f32 {
    if height_m <= 0.0 || species_coefficient <= 0.0 {
        return 0.0;
    }
    let diameter = species_coefficient * height_m.powf(0.67);
    tracing::trace!(
        height_m,
        species_coefficient,
        diameter,
        "height_to_diameter"
    );
    diameter
}

/// Leaf area from height using allometric scaling (m²).
///
/// `leaf_area = coefficient × height^1.5`
///
/// Based on pipe model theory (Shinozaki 1964).
/// Typical coefficients: oak 8.0, bamboo 2.0, grass 0.5.
///
/// - `height_m` — plant height (meters)
/// - `species_coefficient` — dimensionless scaling coefficient
#[must_use]
pub fn height_to_leaf_area(height_m: f32, species_coefficient: f32) -> f32 {
    if height_m <= 0.0 || species_coefficient <= 0.0 {
        return 0.0;
    }
    let area = species_coefficient * height_m.powf(1.5);
    tracing::trace!(height_m, species_coefficient, area, "height_to_leaf_area");
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_balanced_sums_to_input() {
        let pool = allocate(1000.0, AllocationStrategy::Balanced);
        let total = pool.total_kg();
        assert!(
            (total - 1.0).abs() < 0.001,
            "1000g should yield 1.0kg total, got {total}"
        );
    }

    #[test]
    fn allocate_stressed_root_dominant() {
        let pool = allocate(1000.0, AllocationStrategy::StressedRoot);
        assert!(pool.root_kg > pool.leaf_kg);
        assert!(pool.root_kg > pool.stem_kg);
        assert!(pool.root_kg > pool.reproductive_kg);
    }

    #[test]
    fn allocate_reproductive_dominant() {
        let pool = allocate(1000.0, AllocationStrategy::Reproductive);
        assert!(pool.reproductive_kg > pool.leaf_kg);
        assert!(pool.reproductive_kg > pool.stem_kg);
        assert!(pool.reproductive_kg > pool.root_kg);
    }

    #[test]
    fn allocate_zero_carbon() {
        let pool = allocate(0.0, AllocationStrategy::Balanced);
        assert_eq!(pool.total_kg(), 0.0);
    }

    #[test]
    fn allocate_negative_carbon() {
        let pool = allocate(-100.0, AllocationStrategy::Balanced);
        assert_eq!(pool.total_kg(), 0.0);
    }

    #[test]
    fn height_to_diameter_increases() {
        let short = height_to_diameter(5.0, 0.04);
        let tall = height_to_diameter(25.0, 0.04);
        assert!(tall > short);
    }

    #[test]
    fn height_to_diameter_zero() {
        assert_eq!(height_to_diameter(0.0, 0.04), 0.0);
    }

    #[test]
    fn height_to_diameter_negative() {
        assert_eq!(height_to_diameter(-5.0, 0.04), 0.0);
    }

    #[test]
    fn height_to_leaf_area_increases() {
        let short = height_to_leaf_area(5.0, 8.0);
        let tall = height_to_leaf_area(25.0, 8.0);
        assert!(tall > short);
    }

    #[test]
    fn oak_larger_diameter_than_bamboo() {
        let oak = height_to_diameter(20.0, 0.04);
        let bamboo = height_to_diameter(20.0, 0.008);
        assert!(oak > bamboo);
    }

    #[test]
    fn total_kg_sums_correctly() {
        let pool = BiomassPool {
            leaf_kg: 1.0,
            stem_kg: 2.0,
            root_kg: 3.0,
            reproductive_kg: 4.0,
        };
        assert_eq!(pool.total_kg(), 10.0);
    }

    #[test]
    fn oak_preset_stem_heaviest() {
        let oak = BiomassPool::oak();
        assert!(oak.stem_kg > oak.leaf_kg);
        assert!(oak.stem_kg > oak.root_kg);
        assert!(oak.stem_kg > oak.reproductive_kg);
    }

    #[test]
    fn grass_lighter_than_oak() {
        assert!(BiomassPool::grass().total_kg() < BiomassPool::oak().total_kg());
    }
}
