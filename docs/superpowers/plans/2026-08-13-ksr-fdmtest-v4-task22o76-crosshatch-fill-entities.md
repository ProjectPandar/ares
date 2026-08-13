# Task 22O.76 implementation plan

## Validation contract

A source-complete first `Layer::make_fills` slice exists when a graph-native
layer call transforms full grouped CrossHatch surfaces into ordered extrusion
collections with exact Flow metadata, without treating other filler patterns as
CrossHatch output.

## TDD sequence

1. Add `project_slice/tests/fill_entities` as separate test modules. Build a
   prepared graph with one Internal CrossHatch surface and write a RED test for
   the missing `generate_crosshatch_layer` seam and owned entity metadata.
2. Add `project_slice/fill_entities/{types,crosshatch}.rs` plus a small facade.
   Define the minimum owned path/collection/layer types and implement one
   CrossHatch group/ExPolygon pass using `group_fills` and existing
   `fill::cross_hatch`.
3. Add focused RED/GREEN witnesses for empty output, non-CrossHatch rejection,
   source ordering, atomic range error, repeatability, and graph immutability.
4. Capture pinned Orca CrossHatch entity output for selected KSR layers and the
   all-layer aggregate using source-side instrumentation that serializes only
   path geometry, role, `mm3_per_mm`, width, and height. Remove instrumentation
   and restore the Orca tree byte-exact. Add the independent literals as the
   Rust oracle.
5. Run focused and dependent Nextest, workspace Nextest, strict workspace
   all-target/all-feature Clippy, rustfmt, diff/static/LOC checks, and Tier-1
   compilation gates available locally.
6. Update ADR, option parity, and roadmap with exact evidence. Commit with a
   Conventional Commit and push `main` before the next option/function slice.

## Completed evidence

The compile RED proved the graph-native module/seam was missing. GREEN passes
3/3 focused tests. Strict workspace all-target/all-feature Clippy, rustfmt,
diff, and sub-400-LOC gates pass. The focused Internal CrossHatch witness pins
`mm3_per_mm`, width, and height bits and input immutability. A complete KSR
entity oracle is intentionally deferred until the remaining filler classes are
implemented; O76 is explicitly CrossHatch-only.

## Non-goals

No other filler, lifecycle activation, thin/gap fill, ordering, motion, G-code,
public API, fixture branch, fallback, or option parsing outside the prepared
3MF graph.
