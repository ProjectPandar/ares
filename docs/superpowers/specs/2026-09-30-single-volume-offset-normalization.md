# Single-volume 3MF offset normalization

Upstream boundary and Rust destination: ARD-0070, the final single-volume
normalization loop in `Format/bbs_3mf.cpp:2187-2207`.

Port the offset comparison and redistribution without changing world
placement, linear transforms, instance identities or source metadata.
Leave multi-volume import behavior unchanged. The current domain loader
remains only a temporary upstream validation shell.

Exit: public-load tests for shifted single volume, all instances, zero and
near-equal world offsets and multi-volume exclusion; complete sensitive
native prism RED/GREEN with activation proof; counterfactual failure and
original regressions; workspace Nextest, fmt/Clippy and Tier-1 checks;
independent six-axis review/repair; code and docs in one commit/push.
No additional G-code filters, structural source-pinning or legacy fallback.
Concentric consumer and global parity requirements remain open.
