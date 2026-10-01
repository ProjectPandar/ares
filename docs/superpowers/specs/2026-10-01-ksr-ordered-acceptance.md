# KSR ordered acceptance spec

Owning upstream export boundary: `Print.cpp:2618-2638::Print::export_gcode`,
`GCode.cpp:2030::GCode::do_export`, `GCodeWriter`, and
`GCodeProcessor::run_post_process`. Rust destination: existing public CLI/core
project-byte integration tests and browser WASM output comparison. No new
production pipeline behavior is introduced.

Include upgrading CLI/browser acceptance from semantic tolerances and ID
rewrites to complete ordered bytes using the already shared generator-only
comparator. Retain full paired files and the independent initialized capture
identity, preserving both committed native reference streams unchanged.
Delete the obsolete 90% XY coverage test and orphaned ID normalizer. Preserve
semantic tooling as explicitly diagnostic only; no fallback after mismatch.

Acceptance covers actual native and actual browser output, not substituting
native bytes for browser evidence. An object-ID mutation must be rejected
after it is shown to pass the old gate. Any WASM byte difference must
remain RED until fixed at its owning upstream rewrite boundary; do not loosen
the comparator. Global printer/default/option parity is not implied.

Verify separate test modules, Rust files<400LOC, fmt/clippy, workspace Nextest,
relevant Tier-1 checks, retained evidence and independent six-axis review
with fix/re-review. Commit code/docs together and push only after approval.
