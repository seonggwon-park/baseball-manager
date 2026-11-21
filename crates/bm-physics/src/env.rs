//! Atmosphere: air density from temperature, pressure, humidity and elevation,
//! plus a simple wind description.
//!
//! The density formula is the one used in A. M. Nathan's Trajectory Calculator
//! (spreadsheet `TrajectoryCalculator-new-3D`, cells D2 / D20):
//!
//! ```text
//! rho = 1.2929 * 273/(T+273) * (P*exp(-beta*h) - 0.3783*RH*SVP/100) / 760      [kg/m^3]
//! SVP = 4.5841 * exp((18.687 - T/234.5) * T / (257.14 + T))                    [mm Hg]
//! ```
//!
//! with `T` in deg C, `P` the sea-level barometric pressure in mm Hg, `h` the
//! elevation in m, `RH` in percent and `beta = 1.217e-4 /m`. The saturation
//! vapour pressure term is the Buck equation.

use crate::vec3::Vec3;

/// Barometric scale factor, 1/m (Nathan's spreadsheet, cell D5).
pub const BETA: f64 = 1.217e-4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atmosphere {
    /// Air temperature, deg C.
    pub temp_c: f64,
    /// Sea-level-corrected barometric pressure, mm Hg (760 = 29.92 inHg).
    pub pressure_mmhg: f64,
    /// Relative humidity, percent (0..=100).
    pub rel_humidity: f64,
    /// Elevation above sea level, m.
    pub elevation_m: f64,
}

impl Default for Atmosphere {
    /// 21.1 C (70 F), 29.92 inHg, 50 % RH, sea level.
    fn default() -> Self {
        Atmosphere { temp_c: (70.0 - 32.0) * 5.0 / 9.0, pressure_mmhg: 29.92 * 25.4, rel_humidity: 50.0, elevation_m: 0.0 }
    }
}

impl Atmosphere {
    pub fn new(temp_c: f64, pressure_mmhg: f64, rel_humidity: f64, elevation_m: f64) -> Self {
        Atmosphere { temp_c, pressure_mmhg, rel_humidity, elevation_m }
    }

    /// Convenience constructor from Fahrenheit / inHg / feet.
    pub fn from_imperial(temp_f: f64, pressure_inhg: f64, rel_humidity: f64, elevation_ft: f64) -> Self {
        Atmosphere { temp_c: (temp_f - 32.0) * 5.0 / 9.0, pressure_mmhg: pressure_inhg * 1000.0 / 39.37, rel_humidity, elevation_m: elevation_ft / 3.2808 }
    }

    /// Saturation vapour pressure, mm Hg (Buck equation).
    pub fn saturation_vapor_pressure_mmhg(&self) -> f64 {
        let t = self.temp_c;
        4.5841 * ((18.687 - t / 234.5) * t / (257.14 + t)).exp()
    }

    /// Air density, kg/m^3.
    pub fn air_density(&self) -> f64 {
        let t = self.temp_c;
        let svp = self.saturation_vapor_pressure_mmhg();
        let p = self.pressure_mmhg * (-BETA * self.elevation_m).exp();
        1.2929 * (273.0 / (t + 273.0)) * (p - 0.3783 * self.rel_humidity * svp / 100.0) / 760.0
    }
}

/// Uniform horizontal wind acting above `min_height`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wind {
    /// Wind velocity (the direction the air moves toward), m/s.
    pub velocity: Vec3,
    /// Height above which the wind acts, m (0 = everywhere).
    pub min_height: f64,
}

impl Wind {
    pub const CALM: Wind = Wind { velocity: Vec3::ZERO, min_height: 0.0 };

    /// `speed` in m/s blowing toward compass-like `bearing` measured from the
    /// +y axis (home plate -> second base), positive toward +x (first base side).
    /// A bearing of 0 is "blowing out to center field".
    pub fn toward(speed: f64, bearing_rad: f64) -> Self {
        Wind { velocity: Vec3::new(speed * bearing_rad.sin(), speed * bearing_rad.cos(), 0.0), min_height: 0.0 }
    }

    #[inline]
    pub fn at(&self, height: f64) -> Vec3 {
        if height >= self.min_height {
            self.velocity
        } else {
            Vec3::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_matches_nathan_spreadsheet_pitched_sheet() {
        // PitchedBallTrajectory sheet: 70 F, 15 ft, 50 % RH, 29.92 inHg -> D2 = 1.1937672240686363
        let a = Atmosphere::from_imperial(70.0, 29.92, 50.0, 15.0);
        assert!((a.air_density() - 1.193_767_224).abs() < 1e-6, "{}", a.air_density());
        assert!((a.saturation_vapor_pressure_mmhg() - 18.794_410_8).abs() < 1e-6);
    }

    #[test]
    fn density_matches_nathan_spreadsheet_batted_sheet() {
        // BattedBallTrajectory sheets: 75 F, sea level, 50 % RH, 29.92 inHg -> D2 = 1.1822362850090076
        let a = Atmosphere::from_imperial(75.0, 29.92, 50.0, 0.0);
        assert!((a.air_density() - 1.182_236_285).abs() < 1e-6, "{}", a.air_density());
    }

    #[test]
    fn density_drops_with_elevation_and_heat() {
        let sea = Atmosphere::from_imperial(70.0, 29.92, 50.0, 0.0).air_density();
        let denver = Atmosphere::from_imperial(70.0, 29.92, 50.0, 5200.0).air_density();
        let hot = Atmosphere::from_imperial(95.0, 29.92, 50.0, 0.0).air_density();
        assert!(denver < sea * 0.85 && denver > sea * 0.78, "denver/sea = {}", denver / sea);
        assert!(hot < sea);
    }
}
