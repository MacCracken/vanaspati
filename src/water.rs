//! Soil water storage and hydrology — precipitation, infiltration, drainage,
//! root uptake, and water balance for plant-soil systems.

use serde::{Deserialize, Serialize};

/// Soil texture type — determines hydraulic properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SoilType {
    /// Sand: fast drainage, low water holding capacity.
    Sand,
    /// Sandy loam: moderate drainage and retention.
    SandyLoam,
    /// Loam: balanced drainage and retention (ideal for most plants).
    Loam,
    /// Clay loam: slow drainage, good retention.
    ClayLoam,
    /// Clay: very slow drainage, high retention but poor aeration.
    Clay,
}

/// Soil water storage state and hydraulic properties.
///
/// All water volumes are in mm (equivalent to liters/m²).
/// The soil column is defined by a depth in meters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoilWater {
    pub soil_type: SoilType,
    pub depth_m: f32,           // soil column depth (meters)
    pub water_content_mm: f32,  // current water stored (mm)
    pub saturation_mm: f32,     // maximum water at full saturation (mm)
    pub field_capacity_mm: f32, // water held after drainage stops (mm)
    pub wilting_point_mm: f32,  // water unavailable to plants (mm)
}

impl SoilWater {
    /// Create a soil water store from type and depth, initialized at field capacity.
    ///
    /// Hydraulic properties from Saxton & Rawls (2006) pedotransfer functions.
    /// Values are volumetric water content (fraction) × depth (mm).
    ///
    /// | Type | Saturation | Field capacity | Wilting point |
    /// |------|-----------|----------------|---------------|
    /// | Sand | 0.43 | 0.10 | 0.05 |
    /// | SandyLoam | 0.45 | 0.20 | 0.08 |
    /// | Loam | 0.46 | 0.27 | 0.12 |
    /// | ClayLoam | 0.48 | 0.32 | 0.18 |
    /// | Clay | 0.50 | 0.38 | 0.25 |
    ///
    /// - `soil_type` — soil texture classification
    /// - `depth_m` — soil column depth (meters)
    #[must_use]
    pub fn new(soil_type: SoilType, depth_m: f32) -> Self {
        let depth_mm = depth_m * 1000.0;
        let (sat_frac, fc_frac, wp_frac) = match soil_type {
            SoilType::Sand => (0.43, 0.10, 0.05),
            SoilType::SandyLoam => (0.45, 0.20, 0.08),
            SoilType::Loam => (0.46, 0.27, 0.12),
            SoilType::ClayLoam => (0.48, 0.32, 0.18),
            SoilType::Clay => (0.50, 0.38, 0.25),
        };
        Self {
            soil_type,
            depth_m,
            water_content_mm: fc_frac * depth_mm, // start at field capacity
            saturation_mm: sat_frac * depth_mm,
            field_capacity_mm: fc_frac * depth_mm,
            wilting_point_mm: wp_frac * depth_mm,
        }
    }

    /// Sand: 1 meter depth.
    #[must_use]
    pub fn sand() -> Self {
        Self::new(SoilType::Sand, 1.0)
    }

    /// Loam: 1 meter depth (ideal agricultural soil).
    #[must_use]
    pub fn loam() -> Self {
        Self::new(SoilType::Loam, 1.0)
    }

    /// Clay: 1 meter depth.
    #[must_use]
    pub fn clay() -> Self {
        Self::new(SoilType::Clay, 1.0)
    }

    /// Water available to plants (mm). The range between current content and wilting point.
    #[must_use]
    #[inline]
    pub fn available_water_mm(&self) -> f32 {
        (self.water_content_mm - self.wilting_point_mm).max(0.0)
    }

    /// Plant-available water capacity (mm). The range between field capacity and wilting point.
    #[must_use]
    #[inline]
    pub fn available_capacity_mm(&self) -> f32 {
        (self.field_capacity_mm - self.wilting_point_mm).max(0.0)
    }

    /// Relative water content (0.0–1.0). Fraction of plant-available water remaining.
    ///
    /// 1.0 = at field capacity, 0.0 = at wilting point.
    #[must_use]
    pub fn relative_water_content(&self) -> f32 {
        let cap = self.available_capacity_mm();
        if cap <= 0.0 {
            return 0.0;
        }
        let rwc = (self.available_water_mm() / cap).clamp(0.0, 1.0);
        tracing::trace!(
            water_content_mm = self.water_content_mm,
            available = self.available_water_mm(),
            capacity = cap,
            rwc,
            "relative_water_content"
        );
        rwc
    }

    /// Volumetric water content (fraction, 0.0–~0.5).
    #[must_use]
    #[inline]
    pub fn volumetric_water_content(&self) -> f32 {
        let depth_mm = self.depth_m * 1000.0;
        if depth_mm <= 0.0 {
            return 0.0;
        }
        self.water_content_mm / depth_mm
    }

    /// Is the soil saturated? (water content at or above saturation)
    #[must_use]
    #[inline]
    pub fn is_saturated(&self) -> bool {
        self.water_content_mm >= self.saturation_mm
    }

    /// Is the soil at or below wilting point?
    #[must_use]
    #[inline]
    pub fn is_wilted(&self) -> bool {
        self.water_content_mm <= self.wilting_point_mm
    }

    /// Deficit below field capacity (mm). How much rain is needed to refill.
    #[must_use]
    #[inline]
    pub fn deficit_mm(&self) -> f32 {
        (self.field_capacity_mm - self.water_content_mm).max(0.0)
    }

    /// Add water (rain/irrigation) to the soil. Returns excess (runoff) in mm.
    ///
    /// Water fills up to saturation; anything beyond runs off.
    ///
    /// - `water_mm` — water to add (mm)
    pub fn add_water(&mut self, water_mm: f32) -> f32 {
        if water_mm <= 0.0 {
            return 0.0;
        }
        let space = (self.saturation_mm - self.water_content_mm).max(0.0);
        let absorbed = water_mm.min(space);
        let runoff = water_mm - absorbed;
        self.water_content_mm += absorbed;
        tracing::trace!(
            water_mm,
            absorbed,
            runoff,
            content = self.water_content_mm,
            "add_water"
        );
        runoff
    }

    /// Remove water (transpiration/evaporation/drainage). Returns actual removed (mm).
    ///
    /// Cannot remove below zero.
    ///
    /// - `water_mm` — water to remove (mm)
    pub fn remove_water(&mut self, water_mm: f32) -> f32 {
        if water_mm <= 0.0 {
            return 0.0;
        }
        let actual = water_mm.min(self.water_content_mm);
        self.water_content_mm -= actual;
        tracing::trace!(
            water_mm,
            actual,
            content = self.water_content_mm,
            "remove_water"
        );
        actual
    }

    /// Gravity drainage — water above field capacity drains exponentially
    /// toward field capacity. Returns water drained (mm).
    ///
    /// Drainage rate fraction per day (proportional to K_sat, empirically adjusted):
    /// Sand 0.6, SandyLoam 0.4, Loam 0.25, ClayLoam 0.15, Clay 0.08.
    ///
    /// - `fraction_of_day` — time step as fraction of a day (1.0 = full day)
    pub fn drain(&mut self, fraction_of_day: f32) -> f32 {
        let excess = (self.water_content_mm - self.field_capacity_mm).max(0.0);
        if excess <= 0.0 {
            return 0.0;
        }
        let rate = match self.soil_type {
            SoilType::Sand => 0.6,
            SoilType::SandyLoam => 0.4,
            SoilType::Loam => 0.25,
            SoilType::ClayLoam => 0.15,
            SoilType::Clay => 0.08,
        };
        let drained = excess * rate * fraction_of_day.clamp(0.0, 1.0);
        self.water_content_mm -= drained;
        tracing::trace!(
            excess,
            rate,
            fraction_of_day,
            drained,
            content = self.water_content_mm,
            "drain"
        );
        drained
    }
}

/// Saturated hydraulic conductivity (mm/day).
///
/// Rate at which water moves through saturated soil.
/// Determines infiltration capacity and drainage speed.
///
/// Typical values (Rawls et al. 1982):
/// Sand 1500, SandyLoam 600, Loam 250, ClayLoam 80, Clay 20.
#[must_use]
pub fn saturated_conductivity(soil_type: SoilType) -> f32 {
    let k = match soil_type {
        SoilType::Sand => 1500.0,     // mm/day
        SoilType::SandyLoam => 600.0, // mm/day
        SoilType::Loam => 250.0,      // mm/day
        SoilType::ClayLoam => 80.0,   // mm/day
        SoilType::Clay => 20.0,       // mm/day
    };
    tracing::trace!(?soil_type, k, "saturated_conductivity");
    k
}

/// Infiltration rate (mm/hr) using simplified Green-Ampt (Philip approximation).
///
/// `rate = K_sat × (1 + deficit / depth)`
///
/// Cumulative infiltration F is not tracked; suitable for daily timestep models.
///
/// Higher deficit = faster infiltration (dry soil pulls water in).
/// Capped at rainfall rate (can't infiltrate more than falls).
///
/// - `soil_type` — soil texture
/// - `deficit_mm` — soil water deficit below saturation (mm)
/// - `depth_mm` — wetting front depth (mm)
/// - `rainfall_rate_mm_hr` — incoming rain rate (mm/hr)
#[must_use]
pub fn infiltration_rate(
    soil_type: SoilType,
    deficit_mm: f32,
    depth_mm: f32,
    rainfall_rate_mm_hr: f32,
) -> f32 {
    if rainfall_rate_mm_hr <= 0.0 || depth_mm <= 0.0 {
        return 0.0;
    }
    let k_sat_hr = saturated_conductivity(soil_type) / 24.0; // mm/day → mm/hr
    let rate = k_sat_hr * (1.0 + deficit_mm / depth_mm);
    let effective = rate.min(rainfall_rate_mm_hr);
    tracing::trace!(
        ?soil_type,
        deficit_mm,
        depth_mm,
        k_sat_hr,
        rate,
        effective,
        "infiltration_rate"
    );
    effective
}

/// Surface evaporation from bare soil (mm/day).
///
/// `E_soil = E_pot × (water_content / field_capacity)^0.5`
///
/// Evaporation decreases as soil dries (Philip 1957 stage-2 approximation).
///
/// - `potential_evaporation_mm_day` — potential evaporation rate (mm/day)
/// - `water_content_mm` — current soil water (mm)
/// - `field_capacity_mm` — field capacity (mm)
#[must_use]
pub fn soil_evaporation(
    potential_evaporation_mm_day: f32,
    water_content_mm: f32,
    field_capacity_mm: f32,
) -> f32 {
    if potential_evaporation_mm_day <= 0.0 || water_content_mm <= 0.0 || field_capacity_mm <= 0.0 {
        return 0.0;
    }
    let ratio = (water_content_mm / field_capacity_mm).min(1.0);
    let evap = potential_evaporation_mm_day * ratio.sqrt();
    tracing::trace!(
        potential_evaporation_mm_day,
        water_content_mm,
        field_capacity_mm,
        evap,
        "soil_evaporation"
    );
    evap
}

/// Daily water flux summary (all values in mm).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterFluxes {
    pub rainfall_mm: f32,      // incoming precipitation (mm)
    pub infiltration_mm: f32,  // water entering soil (mm)
    pub runoff_mm: f32,        // surface runoff (mm)
    pub drainage_mm: f32,      // gravity drainage below root zone (mm)
    pub transpiration_mm: f32, // water lost through stomata (mm)
    pub evaporation_mm: f32,   // soil surface evaporation (mm)
}

impl WaterFluxes {
    /// Net change in soil water (mm). Positive = gaining, negative = losing.
    #[must_use]
    #[inline]
    pub fn net_change_mm(&self) -> f32 {
        self.infiltration_mm - self.drainage_mm - self.transpiration_mm - self.evaporation_mm
    }
}

/// Run a daily water balance step on a soil column.
///
/// Processes in order:
/// 1. Rainfall → infiltration (excess = runoff)
/// 2. Gravity drainage of water above field capacity
/// 3. Transpiration removal (from stomata)
/// 4. Soil surface evaporation
///
/// Mutates `soil` in place and returns the flux summary.
///
/// - `soil` — mutable soil water state
/// - `rainfall_mm` — daily rainfall (mm)
/// - `transpiration_mm` — daily transpiration demand from canopy (mm)
/// - `potential_evaporation_mm` — daily potential soil evaporation (mm)
pub fn daily_water_balance(
    soil: &mut SoilWater,
    rainfall_mm: f32,
    transpiration_mm: f32,
    potential_evaporation_mm: f32,
) -> WaterFluxes {
    // 1. Rainfall → add to soil, capture runoff
    let runoff = soil.add_water(rainfall_mm.max(0.0));
    let infiltration = rainfall_mm.max(0.0) - runoff;

    // 2. Gravity drainage
    let drainage = soil.drain(1.0);

    // 3. Transpiration (plants extract from available water)
    let transpiration = if transpiration_mm > 0.0 {
        let available = soil.available_water_mm();
        let actual_demand = transpiration_mm.min(available);
        soil.remove_water(actual_demand)
    } else {
        0.0
    };

    // 4. Soil evaporation (from remaining water)
    let evaporation = if potential_evaporation_mm > 0.0 {
        let evap = soil_evaporation(
            potential_evaporation_mm,
            soil.water_content_mm,
            soil.field_capacity_mm,
        );
        soil.remove_water(evap)
    } else {
        0.0
    };

    let fluxes = WaterFluxes {
        rainfall_mm: rainfall_mm.max(0.0),
        infiltration_mm: infiltration,
        runoff_mm: runoff,
        drainage_mm: drainage,
        transpiration_mm: transpiration,
        evaporation_mm: evaporation,
    };

    tracing::trace!(
        rainfall = fluxes.rainfall_mm,
        infiltration = fluxes.infiltration_mm,
        runoff = fluxes.runoff_mm,
        drainage = fluxes.drainage_mm,
        transpiration = fluxes.transpiration_mm,
        evaporation = fluxes.evaporation_mm,
        net = fluxes.net_change_mm(),
        content = soil.water_content_mm,
        "daily_water_balance"
    );

    fluxes
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SoilWater construction ---

    #[test]
    fn loam_default_at_field_capacity() {
        let s = SoilWater::loam();
        assert!((s.relative_water_content() - 1.0).abs() < 0.01);
    }

    #[test]
    fn sand_properties() {
        let s = SoilWater::sand();
        assert!(s.saturation_mm > s.field_capacity_mm);
        assert!(s.field_capacity_mm > s.wilting_point_mm);
        assert!(s.wilting_point_mm > 0.0);
    }

    #[test]
    fn clay_holds_more_than_sand() {
        let sand = SoilWater::sand();
        let clay = SoilWater::clay();
        assert!(clay.field_capacity_mm > sand.field_capacity_mm);
    }

    #[test]
    fn custom_depth() {
        let shallow = SoilWater::new(SoilType::Loam, 0.3);
        let deep = SoilWater::new(SoilType::Loam, 2.0);
        assert!(deep.field_capacity_mm > shallow.field_capacity_mm);
    }

    // --- Available water ---

    #[test]
    fn available_water_at_field_capacity() {
        let s = SoilWater::loam();
        let avail = s.available_water_mm();
        let cap = s.available_capacity_mm();
        assert!(
            (avail - cap).abs() < 0.01,
            "at FC, available should equal capacity"
        );
    }

    #[test]
    fn available_water_at_wilting() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm;
        assert_eq!(s.available_water_mm(), 0.0);
    }

    #[test]
    fn available_water_below_wilting() {
        let mut s = SoilWater::loam();
        s.water_content_mm = 0.0;
        assert_eq!(s.available_water_mm(), 0.0);
    }

    // --- Relative water content ---

    #[test]
    fn rwc_at_field_capacity() {
        let s = SoilWater::loam();
        assert!((s.relative_water_content() - 1.0).abs() < 0.01);
    }

    #[test]
    fn rwc_at_wilting() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm;
        assert_eq!(s.relative_water_content(), 0.0);
    }

    #[test]
    fn rwc_midpoint() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm + s.available_capacity_mm() * 0.5;
        assert!((s.relative_water_content() - 0.5).abs() < 0.01);
    }

    // --- Volumetric ---

    #[test]
    fn volumetric_at_field_capacity_loam() {
        let s = SoilWater::loam();
        let vwc = s.volumetric_water_content();
        assert!((vwc - 0.27).abs() < 0.01, "loam FC = 0.27, got {vwc}");
    }

    #[test]
    fn volumetric_zero_depth() {
        let mut s = SoilWater::loam();
        s.depth_m = 0.0;
        assert_eq!(s.volumetric_water_content(), 0.0);
    }

    // --- Saturation / wilting ---

    #[test]
    fn not_saturated_at_fc() {
        assert!(!SoilWater::loam().is_saturated());
    }

    #[test]
    fn saturated_when_full() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm;
        assert!(s.is_saturated());
    }

    #[test]
    fn not_wilted_at_fc() {
        assert!(!SoilWater::loam().is_wilted());
    }

    #[test]
    fn wilted_at_wp() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm;
        assert!(s.is_wilted());
    }

    // --- Add/remove water ---

    #[test]
    fn add_water_no_overflow() {
        let mut s = SoilWater::loam();
        s.water_content_mm = 200.0; // below saturation
        let runoff = s.add_water(50.0);
        assert_eq!(runoff, 0.0);
        assert!((s.water_content_mm - 250.0).abs() < 0.01);
    }

    #[test]
    fn add_water_overflow() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm - 10.0;
        let runoff = s.add_water(50.0);
        assert!((runoff - 40.0).abs() < 0.01);
        assert!((s.water_content_mm - s.saturation_mm).abs() < 0.01);
    }

    #[test]
    fn add_water_negative() {
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let runoff = s.add_water(-10.0);
        assert_eq!(runoff, 0.0);
        assert_eq!(s.water_content_mm, before);
    }

    #[test]
    fn remove_water_basic() {
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let removed = s.remove_water(50.0);
        assert!((removed - 50.0).abs() < 0.01);
        assert!((s.water_content_mm - (before - 50.0)).abs() < 0.01);
    }

    #[test]
    fn remove_water_cant_go_negative() {
        let mut s = SoilWater::loam();
        s.water_content_mm = 10.0;
        let removed = s.remove_water(50.0);
        assert!((removed - 10.0).abs() < 0.01);
        assert_eq!(s.water_content_mm, 0.0);
    }

    // --- Drainage ---

    #[test]
    fn drain_above_fc() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm; // saturated
        let drained = s.drain(1.0);
        assert!(drained > 0.0, "saturated soil should drain");
        assert!(s.water_content_mm < s.saturation_mm);
    }

    #[test]
    fn drain_at_fc_nothing() {
        let mut s = SoilWater::loam();
        // Already at FC
        let drained = s.drain(1.0);
        assert_eq!(drained, 0.0);
    }

    #[test]
    fn sand_drains_faster_than_clay() {
        let mut sand = SoilWater::sand();
        sand.water_content_mm = sand.saturation_mm;
        let mut clay = SoilWater::clay();
        clay.water_content_mm = clay.saturation_mm;

        let sand_drained = sand.drain(1.0);
        let clay_drained = clay.drain(1.0);
        // Sand drains a higher fraction of excess
        let sand_frac = sand_drained / (sand.saturation_mm - sand.field_capacity_mm + sand_drained);
        let clay_frac = clay_drained / (clay.saturation_mm - clay.field_capacity_mm + clay_drained);
        assert!(sand_frac > clay_frac, "sand should drain faster fraction");
    }

    // --- Deficit ---

    #[test]
    fn deficit_at_fc_is_zero() {
        assert_eq!(SoilWater::loam().deficit_mm(), 0.0);
    }

    #[test]
    fn deficit_when_dry() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm;
        assert!(s.deficit_mm() > 0.0);
    }

    // --- Saturated conductivity ---

    #[test]
    fn sand_most_conductive() {
        assert!(saturated_conductivity(SoilType::Sand) > saturated_conductivity(SoilType::Clay));
    }

    #[test]
    fn all_conductivities_positive() {
        for st in [
            SoilType::Sand,
            SoilType::SandyLoam,
            SoilType::Loam,
            SoilType::ClayLoam,
            SoilType::Clay,
        ] {
            assert!(saturated_conductivity(st) > 0.0);
        }
    }

    // --- Infiltration ---

    #[test]
    fn infiltration_dry_soil_faster() {
        let dry = infiltration_rate(SoilType::Loam, 200.0, 500.0, 50.0);
        let wet = infiltration_rate(SoilType::Loam, 20.0, 500.0, 50.0);
        assert!(dry > wet, "dry soil should infiltrate faster");
    }

    #[test]
    fn infiltration_capped_by_rainfall() {
        let rate = infiltration_rate(SoilType::Sand, 200.0, 500.0, 5.0);
        assert!(
            (rate - 5.0).abs() < 0.01,
            "can't infiltrate more than rainfall"
        );
    }

    #[test]
    fn infiltration_zero_rain() {
        assert_eq!(infiltration_rate(SoilType::Loam, 200.0, 500.0, 0.0), 0.0);
    }

    // --- Soil evaporation ---

    #[test]
    fn evaporation_at_field_capacity() {
        let e = soil_evaporation(5.0, 270.0, 270.0);
        assert!((e - 5.0).abs() < 0.01, "at FC, evap = potential");
    }

    #[test]
    fn evaporation_dry_soil_less() {
        let wet = soil_evaporation(5.0, 270.0, 270.0);
        let dry = soil_evaporation(5.0, 67.5, 270.0); // 25% of FC
        assert!(dry < wet, "dry soil evaporates less");
    }

    #[test]
    fn evaporation_zero_water() {
        assert_eq!(soil_evaporation(5.0, 0.0, 270.0), 0.0);
    }

    #[test]
    fn evaporation_zero_potential() {
        assert_eq!(soil_evaporation(0.0, 270.0, 270.0), 0.0);
    }

    // --- Daily water balance ---

    #[test]
    fn balance_rain_only() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm; // start dry
        let f = daily_water_balance(&mut s, 20.0, 0.0, 0.0);
        assert!((f.rainfall_mm - 20.0).abs() < 0.01);
        assert!((f.infiltration_mm - 20.0).abs() < 0.01);
        assert_eq!(f.runoff_mm, 0.0);
        assert_eq!(f.transpiration_mm, 0.0);
        assert_eq!(f.evaporation_mm, 0.0);
        assert!(s.water_content_mm > s.wilting_point_mm);
    }

    #[test]
    fn balance_heavy_rain_runoff() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm - 5.0; // nearly full
        let f = daily_water_balance(&mut s, 50.0, 0.0, 0.0);
        assert!(f.runoff_mm > 40.0, "most rain should run off");
        assert!(f.infiltration_mm < 10.0);
    }

    #[test]
    fn balance_transpiration_removes_water() {
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let f = daily_water_balance(&mut s, 0.0, 3.0, 0.0);
        assert!(f.transpiration_mm > 0.0);
        assert!(s.water_content_mm < before);
    }

    #[test]
    fn balance_transpiration_limited_by_available() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.wilting_point_mm + 1.0; // barely above wilting
        let f = daily_water_balance(&mut s, 0.0, 100.0, 0.0);
        assert!(
            f.transpiration_mm <= 1.0,
            "can't transpire more than available"
        );
    }

    #[test]
    fn balance_evaporation_removes_water() {
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let f = daily_water_balance(&mut s, 0.0, 0.0, 5.0);
        assert!(f.evaporation_mm > 0.0);
        assert!(s.water_content_mm < before);
    }

    #[test]
    fn balance_drainage_from_saturation() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm;
        let f = daily_water_balance(&mut s, 0.0, 0.0, 0.0);
        assert!(f.drainage_mm > 0.0, "saturated soil should drain");
    }

    #[test]
    fn balance_no_drainage_at_fc() {
        let mut s = SoilWater::loam();
        // At field capacity, no inputs
        let f = daily_water_balance(&mut s, 0.0, 0.0, 0.0);
        assert_eq!(f.drainage_mm, 0.0);
    }

    #[test]
    fn balance_net_change_sign() {
        let mut s = SoilWater::loam();
        // Rain only → net positive
        let f = daily_water_balance(&mut s, 20.0, 0.0, 0.0);
        assert!(f.net_change_mm() > 0.0);

        // Transpiration only → net negative
        let mut s2 = SoilWater::loam();
        let f2 = daily_water_balance(&mut s2, 0.0, 5.0, 3.0);
        assert!(f2.net_change_mm() < 0.0);
    }

    #[test]
    fn balance_negative_rain_ignored() {
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let f = daily_water_balance(&mut s, -10.0, 0.0, 0.0);
        assert_eq!(f.rainfall_mm, 0.0);
        // Only drainage might change it
        assert!(s.water_content_mm <= before);
    }

    #[test]
    fn balance_multi_day_dry_down() {
        let mut s = SoilWater::loam();
        // Simulate 30 dry days with transpiration
        for _ in 0..30 {
            daily_water_balance(&mut s, 0.0, 5.0, 2.0);
        }
        assert!(
            s.relative_water_content() < 0.3,
            "30 dry days should significantly deplete soil, rwc={}",
            s.relative_water_content()
        );
    }

    #[test]
    fn balance_conservation() {
        // Water in = water out + change in storage
        let mut s = SoilWater::loam();
        let before = s.water_content_mm;
        let f = daily_water_balance(&mut s, 15.0, 3.0, 2.0);
        let after = s.water_content_mm;
        let storage_change = after - before;
        let balance = f.infiltration_mm - f.drainage_mm - f.transpiration_mm - f.evaporation_mm;
        assert!(
            (storage_change - balance).abs() < 0.1,
            "water balance should be conserved: storage_change={storage_change}, flux_balance={balance}"
        );
    }

    #[test]
    fn balance_saturated_drain_and_transpire() {
        let mut s = SoilWater::loam();
        s.water_content_mm = s.saturation_mm;
        let before = s.water_content_mm;
        let f = daily_water_balance(&mut s, 0.0, 5.0, 2.0);
        assert!(f.drainage_mm > 0.0, "should drain excess");
        assert!(f.transpiration_mm > 0.0, "should also transpire");
        assert!(s.water_content_mm < before);
        // Conservation still holds
        let change = s.water_content_mm - before;
        let flux = -f.drainage_mm - f.transpiration_mm - f.evaporation_mm;
        assert!((change - flux).abs() < 0.1);
    }
}
