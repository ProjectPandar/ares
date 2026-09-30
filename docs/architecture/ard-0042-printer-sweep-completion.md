# ARD-0042: Printer sweep completion requires every reference comparison

The printer matrix validates the source-cited rewrite of
`OrcaSlicer/src/libslic3r/PrintConfig.cpp` printer/process settings through
`Print::process` and `GCode::do_export`. Its acceptance boundary is the
existing CLI harness `tests/orca_parity/smoke/sweep.rs`, not a new slicer.

Only a nonempty all-PASS matrix completes the sweep. Reference-binary errors
and missing vendor presets explain missing evidence; they cannot count as
successful verification. Preserve their distinct statuses and persist the
summary and existing full artifacts before failing.

Do not alter baseline overrides, denominator enumeration, comparator,
normalization or oracle to obtain a passing result. This correction includes
only completion semantics; untouched-profile and unsupported upstream behavior
remain deferred implementation work. The modified classic-wall matrix remains
limited evidence, and existing Ares scaffold is only a temporary compatibility
shell wherever the upstream behavior is not yet rewritten.
