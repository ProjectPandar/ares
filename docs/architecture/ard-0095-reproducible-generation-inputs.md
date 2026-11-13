# ARD-0095: Reproducible generation inputs

The user requires identical Ares inputs to produce identical complete bytes.
Core `GenerationMetadata` and `SlicerContext` remain explicit execution inputs;
CLI and browser adapters use Unix epoch zero and seed zero instead of reading
the wall clock. This intentionally changes the adapters' default time policy,
not the slicing algorithms. The automatic producer line uses the fixed marker
defined by the [generated-at header contract](gcode-generated-at-header.md),
not an execution time.

Upstream boundaries are `GCode.cpp` header emission and file-start template
timestamp setup (`2549–2581`), and `PlaceholderParser::update_timestamp`
(`PlaceholderParser.cpp:78–101`). Rust destinations are `generation.rs`,
`ares-cli/src/main.rs` and `ares-wasm/src/lib.rs`. The existing timestamp and
seed plumbing is retained, not replaced by a new slicing pipeline.

Ares-written ZIP members explicitly use DOS epoch `1980-01-01 00:00:00`,
because DOS timestamps cannot represent Unix epoch zero. Preserve compression,
member order and contents; retain deliberately varied timestamp fixtures that
test input-container invariance. Official Orca archives remain unmodified.
Currently ZIP writers exist only in test fixtures, not production adapters.

Included: deterministic adapter time and explicit ZIP defaults. Deferred:
unproven cross-platform/worker-count equivalence and any newly observed
ordering instability. Fixed timestamps alone do not establish those claims,
nor excuse real Ares parity differences or relax complete-output comparisons.

## Fixed automatic producer identity

The [generated-at header contract](gcode-generated-at-header.md) replaces the
former undated-header policy and its rejection of dated Ares headers. It ports
`GCode::do_export`'s producer syntax while using exactly
`1980-01-01 at 00:00:00` instead of Native's local clock. Only that complete
fixed Ares line is accepted; arbitrary Ares dates are not normalized.

This marker does not change core metadata or template variables.
`PlaceholderParser::update_timestamp` defines `timestamp`, `year`, `month`,
`day`, `hour`, `minute`, `second`; it emits no G-code itself. Retain all seven
variables and explicit core metadata, defaulting to epoch0, with seed0 unchanged.
Do not rewrite vendor/user templates or suppress their output. Preserve the
BTT header omission, existing vendor normalization scope and every non-producer
byte assertion.
