//! Shared scalar transcendental routines used by the slicer.

pub(crate) mod acosf;
mod atan2_f64;
mod sin_f64;
pub(crate) mod sincosf;

pub(crate) use atan2_f64::atan2_f64;
pub(crate) use sin_f64::sin_f64;
