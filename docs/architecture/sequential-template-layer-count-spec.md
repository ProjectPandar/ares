# Sequential template total layer count — SPEC

## Upstream boundary

Pinned OrcaSlicer `8500fcdcc` `src/libslic3r/GCode.cpp::_do_export:2484–2528`
sets `m_layer_count` by print mode: sequential printing accumulates each
object's merged object/support layers per instance; ByLayer counts globally
merged print-z once. At `GCode.cpp:2890`, that same `m_layer_count` supplies the
`total_layer_count` placeholder. `GCode.cpp:4677–4684` consumes it in layer
templates. This is not a new Ares-owned pipeline or counter design.

Rust destination is the existing private
`ares-core/src/project_slice/gcode_emit/orchestration.rs::emit` initializer of
`EmitState.total_layer_count`. The existing `tail_layer_count` already selects
the mode-dependent total; the initializer currently passes `sliced_layer_count`
instead. That initializer is an incomplete upstream compatibility shell.

## Included behavior and later GREEN bound

Pass the existing `tail_layer_count` into `EmitState.total_layer_count`.
Do not add a counter or change its computation, scheduling, instance traversal,
layer numerators, placeholder evaluator, stock templates, or formatting.
Only that field initializer may change in later GREEN, with frozen RED tests.
Keep file-start/header counts, compatible-tail code, role caches, geometry,
writer, processor and M73 placement untouched. No tokenizer/fan/zero-E,
jerk/format/acceleration or `value.rs` changes.

## Public Native-first E2E seam and frozen witnesses

Use unchanged `native_stream_case::check`: two independent fresh Native slices,
complete generator-only replay comparison, callback assertions on both Native
streams, then public byte API and actual CLI with the unchanged complete
`golden::compare_ordered_bytes_generator_only` comparator. Require API/CLI
success and raw byte equality; never invert the target assertion for RED.
No separate or duplicate preflight test, reference editing, subsection-only
acceptance, arbitrary omission, or hand-authored expected G-code.

| Case | Native total | RED expectation |
| --- | --- | --- |
| Stock Bambu Lab A1, two objects, ByObject | 81 | FAIL: Rust uses denominator 50 |
| Same stock A1 scene, ByLayer | 50 | complete PASS |
| Original frozen P1S ByLayer Auto-lift scene | 50 | complete PASS |

A1 target input SHA256:
`b436ba663498652206663fb1f276c7d972f09b5b6fccad15e1d998aeeb53e138`.
Original P1S input SHA256:
`f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd`.
The A1 ByLayer input is copied byte-for-byte from discovery and frozen
with its SHA256 in the test/fixture README. All use real stock layer templates,
the unchanged overhang/hole dual-object geometry and labels 45/56. No synthetic
template or geometry is introduced. The P1S fixture is reused, not regenerated.

Discovery A1 target has first difference byte 55433 / line 1278: Native
`; layer num/total_layer_count: 1/81`, Rust `1/50`. The exhaustive raw difference
is generator identity plus 81 template denominators only. A1/P1S ByLayer
complete comparisons pass. Formal RED must reproduce, not rely on discovery.

## Failure modes and exit criteria

- Wrong fixture/config/geometry: reject on frozen SHA, stock printer/sequence,
  zero extruder offset, real object labels/names and full Native layer sequence.
- Native instability or missing witness: fail before public generation.
- API/CLI failure or unequal raw streams: not a genuine count-only RED.
- Compound differences: retain the complete comparison; stop and request review,
  never mask motion/timing/M73 differences to obtain a count-only result.
- Wrong selection/cache: record exact selected tests and commit, and use fresh
  mode-specific caches.

Run directed official and seeded modes serially on exact RED. Each mode must
select exactly three tests: one genuine target failure and two control passes.
Retain both Native replays, public streams, complete comparisons and commands.
Run rustfmt and strict workspace/all-target Clippy. Later GREEN must pass the identical matrix and comparator,
with raw API==CLI and no unrelated production diff. Linux dual-mode runtime
evidence is not Windows/macOS/browser-WASM runtime qualification; the eventual
private scalar handoff introduces no platform API or file/terminal behavior.

## Deferred adjacent upstream behavior

P1S ByObject is a compound residual (count, 2 mm Y transition and M73 relocation),
not this count-only witness. The broader P1S skirt control also remains deferred.
Do not change either to widen this slice. Defer file-start/role-cache count
consumers, multi-tool/support/raft/variable-height/copy qualification and other
runtime paths not demonstrated by these three immutable fixtures.

[Non-negotiable decisions](ard-sequential-template-layer-count.md).
