//! Shared scalar binary32 transcendental routines used by the slicer.

pub(crate) mod acosf;
mod atan2_f64;
pub(crate) mod sincosf;

pub(crate) use atan2_f64::atan2_f64;
