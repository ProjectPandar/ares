use super::*;

#[test]
fn fresh_runtime_controls_do_not_leak_raw_group_phase_into_other_fillers() {
    for phase in [-20_000.5, 20_000.5] {
        for symmetric in [false, true] {
            for pattern in [
                ProcessInfillPattern::Rectilinear,
                ProcessInfillPattern::AlignedRectilinear,
                ProcessInfillPattern::Monotonic,
                ProcessInfillPattern::MonotonicLine,
            ] {
                assert_eq!(runtime_controls(pattern, phase, symmetric), (0, false));
            }
            assert_eq!(
                runtime_controls(ProcessInfillPattern::ZigZag, phase, symmetric),
                (0, symmetric)
            );
            assert_eq!(
                runtime_controls(ProcessInfillPattern::CrossZag, phase, symmetric),
                (phase as i64, symmetric)
            );
        }
    }
}
