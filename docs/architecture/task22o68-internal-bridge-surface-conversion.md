# Task 22O.68 architecture decision record

## Status

Accepted and implemented.

## Upstream boundary

Task 22O.68 ports pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:3352-3367`, which converts the current layer's
  committed bridge candidates into internal-bridge surfaces; and
- `src/libslic3r/Surface.hpp:14-30,105-113`, which assigns
  `stInternalBridge = 6` and classifies it as a bridge.

## Decision

A crate-private, production-unwired operation receives the current region index,
borrowed region fill surfaces, and borrowed current-layer O64 candidates. It:

1. traverses candidates in composer append order;
2. retains only candidates belonging to the current region;
3. resolves Orca's `original_surface` identity through Ares's stable
   `CandidateSource.surface_index`;
4. requires the resolved source to be `InternalSolid`;
5. calls the existing default-NonZero `union_ex` exactly once for that matched
   candidate;
6. clones every source metadata field, replaces only the ExPolygon, retags it
   `InternalBridge`, replaces its bridge angle, and emits engine results in
   returned order.

Unmatched region/index/kind candidates and empty unions emit nothing. The first
geometry error is returned without mutating borrowed inputs or exposing partial
output.

## Boundaries

The operation returns fresh owned surfaces and adds no option inference, region
mutation, composer, prepared successor, lifecycle activation, filesystem or
platform behavior. Solid recomposition at `PrintObject.cpp:3368+`, extrusion,
motion, G-code, CLI, and full golden parity remain deferred.

All Rust source and test files stay below 400 physical lines and use ordinary
modules; `include!` and `include_bytes!` are not used for source splitting.

## Evidence

Behavioral RED was preserved in `/tmp/task22o68-behavioral-red.log`.
Implementation verification passed:

- focused Task 22O.68: 6/6;
- dependency band through O68 plus geometry/Flow: 788/788;
- workspace: 6,448/6,448, with two skipped;
- warning-denying workspace Clippy, rustfmt, wasm32 core/WASM, and x86_64/aarch64
  Windows and macOS checks;
- 14/14 compiling behavioral mutations killed, with production restored to
  SHA-256 `d8f2e21dccc653c867bbaf5950061a264589a5b2f007b4b373686e2d2e21290b`;
- mutation output SHA-256
  `ec38c69e2fab03dafbe624bf25c643424aeefebb6a1cafa7b73162e0430ea560`.

Independent six-axis implementation review approved with no repair items.
