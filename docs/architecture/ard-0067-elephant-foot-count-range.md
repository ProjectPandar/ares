# ARD-0067: Elephant-foot compensation count range parity

Upstream boundary: `PrintConfig.cpp:748-758` declares the scalar
`elefant_foot_compensation_layers` minimum 1; `ConfigOptionInt`
(`Config.hpp:955-983`) permits INT_MAX without a nil sentinel.
`PrintObjectSlice.cpp:1246-1270` bounds backup allocation by actual layer count,
but evaluates the compensation ramp using the original requested count.
Rust destination is `project_slice/compensation/preflight.rs`.
`Fill/Fill.cpp:1346-1352` separately adjusts solid-fill density;
`group_fills/params/projection.rs` owns that rewrite.

Include min 1 / INT_MAX / full-range fixed-seed interior, exact original
export application, final config and first-reference complete ordered output.
Activate positive compensation 0.15 with no raft; keep solid-fill density
100% to isolate geometric compensation. Compare against a zero-compensation
control to prove activation, without using a motion-only parity filter.

Do not clamp the request to fixture size, allocate by requested count, add
fallbacks or loosen comparison. Existing native fixtures are temporary adapters
to upstream config and byte-output boundaries, not an independent pipeline.
Defer reduced solid-fill density, raft and multi-region cross-products.
Repair production only if actual original output establishes a discrepancy.
