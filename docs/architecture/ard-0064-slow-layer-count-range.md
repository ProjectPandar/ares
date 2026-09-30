# ARD-0064: Scalar slow-layer count range

Upstream owner: `PrintConfig.cpp:3426-3434` declares `slow_down_layers` as
`coInt`, minimum 0, with no explicit maximum. `Config.hpp:955-983`
(`ConfigOptionInt`) stores/serializes a signed C++ int; unlike
`ConfigOptionIntsTempl` (`Config.hpp:995-1016`), this scalar does not reserve
INT_MAX for nil. `ConfigOptionDef` defaults its upper bound to FLT_MAX
(`Config.hpp:2475-2479`), above the scalar's representable INT_MAX ceiling.

`GCode.cpp:6572-6585` applies the no-raft early-layer linear speed ramp using
layer index divided by this count. The Rust production destination already
exists in `project_slice/gcode_emit/motion/features.rs::slow_down_layer_ramp`
and `MotionOptions`; this slice validates that rewrite rather than designing
another speed pipeline. Native fixture/domain scaffolding remains a temporary
adapter around these source-owned option and export contracts.

Include exact 0 / INT_MAX / fixed-seed full-range interior, plus 1 (ramp off)
and 4 (short active ramp). Generate these in `option_coverage/domains.rs` and
test in a separate `slow_layer_count.rs` module. Require original exported
requested/effective values, final config literals, first-reference complete
ordered original/Ares output and retained artifacts. A motion-line comparison
between zero and active counts must additionally prove the fixture exercises
the ramp; this is an activation assertion, never a relaxed parity comparator.

Defer raft-relative ramp cross-products (`GCode.cpp:6586-6595`), multi-printer
combinations and other implicit integer ranges. Do not assume vector nil rules
apply to scalars; do not clamp the requested count to fixture height. Production
changes require an observed output discrepancy at the cited rewrite boundary.
