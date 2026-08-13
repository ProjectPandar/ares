# Task 22O.68 — internal bridge surface conversion

## Source boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/PrintObject.cpp:3352-3367`, together with the directly owned
`stInternalBridge = 6` and bridge predicate vocabulary in
`src/libslic3r/Surface.hpp:14-30,105-113`.

## Required behavior

The private operation consumes no new raw option surface. Given the current
region index, borrowed region fill surfaces, and borrowed current-layer O64
candidate history, it must:

- preserve candidate append order;
- use `CandidateSource.region_index` and stable `surface_index` as Ares's
  equivalent of Orca's region loop and `original_surface` pointer identity;
- accept only `InternalSolid` sources;
- call the existing default-NonZero `union_ex` once per accepted candidate;
- preserve every source metadata field except surface kind and bridge angle;
- produce one fresh `InternalBridge` surface for each union result, in engine
  result order, with the candidate bridge angle;
- emit nothing for unmatched region/index/kind or an empty union;
- return the first union error atomically, without mutating any borrowed input.

## Included and deferred

Included: enum discriminant/predicate, matching, default union, metadata copy,
kind/angle replacement, owned ordered output, and error propagation.

Deferred: `PrintObject.cpp:3368+` solid recomposition, region replacement,
second pass, transaction composer, prepared successor/lifecycle, extrusion,
motion, G-code, CLI, and complete KSR golden parity.

## Acceptance

Tests must discriminate region/index/kind selection, candidate order, exact
metadata and angle replacement, empty/unmatched paths, multi-result engine order,
real union/range failure, first-error precedence, and complete borrowed-input
nonmutation. The surface-kind test must freeze discriminant 6 and bridge
classification.

Mutation acceptance must kill region/index/kind bypass, reversed traversal,
union bypass/repetition, EvenOdd substitution, swallowed errors, output sorting,
wrong kind, preserved angle, and default-metadata substitution. All mutations
must compile and production must be restored byte-exactly.

Final gates: focused and dependency Nextest, workspace Nextest, warning-denying
Clippy, rustfmt, wasm32 core/WASM, x86_64/aarch64 Windows and macOS checks,
diff/LOC/static/include/pinned-Orca/no-staged audits, followed by independent
read-only six-axis review and repair/re-review until approval.
