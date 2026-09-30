# ARD-0053: Resolution normalization evidence

Upstream `PrintConfig.cpp:5172-5179` defines `resolution` with minimum zero,
default 0.01 and no explicit maximum. `Config.hpp:2475-2479` supplies the
implicit FLT_MAX ceiling. `DynamicPrintConfig::normalize_fdm`
at 8584-8586 and `normalize_fdm_1` at 8681-8683 clamp it to at least 0.001.
The Rust destination already exists in
`ares-core::options::project_fdm_normalization::stage1` (and the older
`fdm_normalization` seam). This slice repairs the CLI application-evidence
shell around that source-owned normalization, not the slicing pipeline.

Preserve the raw requested value, verify the exact expected exported value
and record the source rule for nonnegative requests below 0.001. Do not
accept arbitrary value mismatches or change the full ordered byte comparator.
At and above the floor, retain ordinary equality checks.

Include zero, below-floor, threshold, above-floor and seeded generated
requests through original export and full G-code comparison, plus negative
proof tests. Defer raw-3MF normalization combinations and model-dependent
resolution/simplification activation. The implicit ceiling and a seeded
interior spanning that ceiling remain unverified: mark this domain INCOMPLETE,
not PASS based on the existing zero/small-interior probes. Do not generalize
this key-specific correction into a broad numeric-domain redesign here.
Existing harness scaffolding remains a temporary validation shell around
the named libslic3r rewrite boundaries.
