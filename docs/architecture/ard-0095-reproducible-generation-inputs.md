# ARD-0095: Reproducible generation inputs

The user requires identical Ares inputs to produce identical complete bytes.
Core `GenerationMetadata` and `SlicerContext` remain explicit execution inputs;
CLI and browser adapters use Unix epoch zero and seed zero instead of reading
the wall clock. This intentionally changes the adapters' default time policy,
not the source-owned header/template formatting or slicing algorithms.

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
