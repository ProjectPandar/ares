# PrintApply LayerRanges assign normalization Spec

## Goal
Port OrcaSlicer's `LayerRanges::assign(...)` interval normalization into `ares-core` as a private staging helper for later PrintApply model-object update work.

## Rewrite gate mapping

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:342-356`: `LayerRanges` and `LayerRange` storage context.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:358-383`: full `LayerRanges::assign(...)` body.
- `OrcaSlicer/src/libslic3r/libslic3r.h:52`: `EPSILON = 1e-4` tolerance used by the assign comparisons.

Deferred context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:385-395`: `LayerRanges::config(...)` lookup is source context only and must remain deferred.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:407+`: `ModelObjectStatus` and model-object apply logic are out of scope.

## Approval gate
Do not begin tests, implementation, or any code changes for M272 until this M272 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Represent input layer config ranges with `start`, `end`, and a lightweight `config_id` integer; this mirrors `const DynamicPrintConfig*` identity without porting config ownership.
- Represent normalized ranges with `start`, `end`, and `Option<config_id>`; `None` mirrors upstream `config { nullptr }`.
- Implement a private helper that receives sorted input ranges and returns normalized ranges in upstream order.
- Preserve `last_z = 0.0` initialization.
- Skip any input range whose `end <= last_z`.
- Clamp each considered input start with `min_z = max(start, 0.0)`.
- Insert a `None` gap `[last_z, min_z]` before the configured range only when `min_z > last_z + EPSILON`, where `EPSILON` is Orca `1e-4`.
- Insert a configured range `[last_z, end]` only when `end > last_z + EPSILON`, where `EPSILON` is Orca `1e-4`, using the input range's config id.
- After all input ranges, return `[0.0, f64::MAX]` with `None` if no ranges were produced.
- If the last produced range has `None`, extend only that range's end to `f64::MAX`.
- If the last produced range has a config id, append a trailing `None` range `[last_end, f64::MAX]`.
- Do not validate sorting, finite floats, or overlap at this private internal boundary; upstream expects sorted input and uses asserts elsewhere.
- Do not implement `LayerRanges::config(...)`, `DynamicPrintConfig`, `ModelConfig`, public APIs, profile loading, UI behavior, slicing, extrusion, G-code, new crates, dependencies, or independent Ares pipeline behavior.

## Non-goals
- No lookup/query helper for desired ranges.
- No config object storage or cloning.
- No model-object status database or apply logic.
- No public API or pipeline wiring.
