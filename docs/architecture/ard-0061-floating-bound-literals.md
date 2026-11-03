# ARD-0061: Floating bound literals in the native parity domain adapter

Source boundary: `libslic3r/PrintConfig.cpp` declares suffixed decimal bounds
for `default_junction_deviation` (3298-3306),
`machine_max_junction_deviation` (4570-4577), `print_flow_ratio` (2327-2337),
`wall_maximum_resolution` (7242-7251), and `wall_maximum_deviation`
(7253-7263). `Config.hpp:2475-2479` stores numeric limits as float;
`Config.cpp:321-337`, `ConfigOptionDef::is_value_valid`, defines tolerant
validation. `PrintConfig.cpp:10534-10555` uses it for scalar/vector values.

The shared parser also reaches `fuzzy_skin_point_distance` (3571-3579),
`fuzzy_skin_scale` (3637-3645), `fuzzy_skin_persistence` (3656-3663), and
`tree_support_tip_diameter` (6496-6505). Include their generated-domain and
original-binary regressions; existing unsupported/nondeterministic output
limitations remain incomplete, not excused as successful comparisons.

Rust destination is the existing native `option_coverage/bounds.rs` source
domain projection. This is a temporary test adapter around upstream schema
semantics, not an independent slicing pipeline or production config rewrite.

Include complete decimal/scientific floating literal recognition, including
the upstream `f` suffix, and regression coverage for the five affected option
domains. Keep arithmetic/identifier expressions unsupported; never reinstate
prefix-only parsing. Sampling must use declared source bounds, with original
requested/effective checks and complete ordered output comparison establishing
wire behavior. Do not silently round, clamp, or waive rejected samples.

Defer arbitrary C++ constant-expression evaluation, new implicit-endpoint
domains, support generation and any claim of testing every adjacent floating
representable value. The original validator's tolerance and binary float
storage must not be confused with exact decimal arithmetic.

Evidence: immutable d2949771 full scan ended with 369 PASS / 279 INCOMPLETE /
2 FAIL, 1638 generated, 1475 strictly compared and 9 rejected (1492.297s).
Its results predate ARD-0060. Five-key reproduction retained 8 rejections in
`${TMP_ROOT}/ares-bound-reject-repro.2y5hhO` (5.657s); single-key minimization in
`${TMP_ROOT}/ares-bound-reject-minimal.tl0PxT` (3.289s) rejects 0.5 and 3.568 against
the original junction-deviation [0, 0.3] bounds. Requested process JSON agrees
with the error, and source declares 0.f/0.3f, excluding injection overwrite
and source/binary schema disagreement for that minimized case.
