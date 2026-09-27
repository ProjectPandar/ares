//! Hilbert-curve pattern flag for the suppress-wipe deretract path
//! (`GCode.cpp:499-502`): bottom/top surfaces filled with
//! `HilbertCurve` suppress the wipe's final retract.

pub(super) fn is_hilbert(pattern: crate::ProcessInfillPattern) -> bool {
    pattern == crate::ProcessInfillPattern::HilbertCurve
}
