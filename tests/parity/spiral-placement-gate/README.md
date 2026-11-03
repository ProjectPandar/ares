# Spiral perimeter placement gate RED

Source: GCode.cpp::GCode::extrude_loop5769–5775, both configured
!spiral_mode AND description==perimeter. SPECf477f9b3c maps the latter
to existing Rust Perimeter-collection/Loop-only dispatch. No production
change is part of RED; GREEN awaits COORD review, FULL awaits window.

These three archives are INPUTS only, not embedded expected outputs:
actual official CLI exports copied unchanged. manifest.json pins whole
3MF/per-ZIP-entry SHA256, retained settings and original export paths.
Control/positive differ only project_settings.config/spiral_mode0→1.
CW-axis rectangle is a regression control, not the sole-key octagon pair:
it must remain passing after its old spiral shell is replaced by the
same general upstream gate. No cylinder/rectangle-specific new exception.

Tests directly call main's unchanged `runner/native_stream_case.rs::check`
with namespace `spiral-placement-gate`, upstream `["GCode.cpp:5769-5775"]`
and assert_two_tools=false. The shared helper owns BOTH Native invocations,
ordered repeat normalization/comparison, native-replay.json and subsequent
NativeCase full validated API/CLI comparison. This case module only passes
frozen inputs/SHA; it duplicates no replay, JSON or comparison mechanics.
The shared helpers retain both Native input/output/launch roots and both
public attempts/comparison.json even when positive fails. No extra Native
content assertions: effective settings are frozen by input SHA, and the
strict full comparison itself proves the positive discrepancy. No M73,
time/config/E/line-ending mask, manual runner, helper copy, borrowed gate,
ignored/expected-failure test.

Targeted nextest expression: `test(spiral_placement_gate_)`.
Each official/seeded mode must show two passing controls and a real
failing positive, not a converted expected failure. Settings and Native motion are not separately
asserted by this runner; SHA freezes input settings, full output comparison
is the parity oracle. The shared check requires configured Native.

External discovery: ${COORD_ROOT}/evidence/w5-spiral-placement-gate/.
Read-only PG equality, pre-filter divergence and0vs6 placement calls are
discovery proof, not a private seam used by these public tests. Inputs
may be sliced independently with `$ORCA --slice 0 --outputdir /tmp/vase positive.3mf`.
No production arithmetic, movement or processor flag fix belongs in RED.
