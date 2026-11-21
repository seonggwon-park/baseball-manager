//! Unit conversions. Everything inside the engine is SI (m, s, kg, rad).
//! Baseball conventions (mph, ft, rpm, oz) only appear at the edges.

use std::f64::consts::PI;

/// Standard gravity, m/s^2 (32.174 ft/s^2, the value used by Nathan's Trajectory Calculator).
pub const G: f64 = 9.806_65;

pub const MPH: f64 = 0.447_04; // m/s per mph
pub const KMH: f64 = 1.0 / 3.6; // m/s per km/h
pub const FT: f64 = 0.304_8; // m per ft
pub const INCH: f64 = 0.025_4; // m per in
pub const OZ: f64 = 0.028_349_523_125; // kg per oz
pub const RPM: f64 = PI / 30.0; // rad/s per rpm
pub const DEG: f64 = PI / 180.0; // rad per degree

#[inline]
pub fn mph(v: f64) -> f64 {
    v * MPH
}
#[inline]
pub fn to_mph(v: f64) -> f64 {
    v / MPH
}
#[inline]
pub fn kmh(v: f64) -> f64 {
    v * KMH
}
#[inline]
pub fn to_kmh(v: f64) -> f64 {
    v / KMH
}
#[inline]
pub fn ft(x: f64) -> f64 {
    x * FT
}
#[inline]
pub fn to_ft(x: f64) -> f64 {
    x / FT
}
#[inline]
pub fn inch(x: f64) -> f64 {
    x * INCH
}
#[inline]
pub fn to_inch(x: f64) -> f64 {
    x / INCH
}
#[inline]
pub fn rpm(w: f64) -> f64 {
    w * RPM
}
#[inline]
pub fn to_rpm(w: f64) -> f64 {
    w / RPM
}
#[inline]
pub fn deg(a: f64) -> f64 {
    a * DEG
}
#[inline]
pub fn to_deg(a: f64) -> f64 {
    a / DEG
}
