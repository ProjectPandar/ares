# Print-wide brim flow source rewrite SPEC

## Upstream boundary and non-negotiable decision

Port OrcaSlicer `libslic3r/Print.cpp:2007-2025::Print::brim_flow`
(upstream revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`).
This is a Print-global flow, not an object-local flow:

1. Start with `m_config.initial_layer_line_width`.
2. If nonpositive, use `m_print_regions.front()->config().inner_wall_line_width`.
3. If still nonpositive, use `m_objects.front()->config().line_width`.
4. Resolve nozzle diameter through the first print region's
   `outer_wall_filament_id - 1`, using source `get_at` semantics.
5. Construct `Flow::new_from_config_width(frPerimeter, width, nozzle,
   skirt_first_layer_height())`; `Print.cpp:2002-2005` defines that height as
   `initial_layer_print_height`.

`Brim.cpp:445-463` consumes `print.brim_flow()` and derives the shared scaled
spacing/flow-width lattice. The future Rust destination is the existing flow
preparation in `crates/ares-core/src/project_slice/gcode_emit/brim.rs:113-148`
and its callers, not an Ares-designed slicing stage. That preparation currently
chooses each object's first candidate model-part region and object options;
it is a temporary compatibility shell to be replaced by the cited Print-global
selection. Do not introduce a second competing flow definition.

## Included and deferred behavior

The intended future rewrite includes the global width fallback chain, first
print-region external-wall nozzle selection, first object default width and
existing Flow construction/scaled spacing semantics. It must preserve upstream
print-region/object ordering, not substitute the currently emitting object or
its region. Before any GREEN implementation, audit the existing Rust carriers
for that order and report required mechanical wiring for approval.

Deferred: independent wall/perimeter/infill flow algorithms, skirt flow,
painted-brim behavior, brim owner ordering/contact components, support/tower
algorithms, travel, cooling/time estimation, new validation and GUI changes.
Do not broaden into those areas to conceal a strict residual. Existing scaffold
is only a compatibility shell around the upstream flow concept, not the owner
of independently designed per-object brim semantics.

## Frozen GUI envelope and two-input RED

Base this slice on main `ad5111f0b`. Seed is existing
`tests/parity/ordered-brim-owner/far-uncombined.3mf`, SHA256
`c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6`.
Retain both objects, meshes, transforms, instance placement and all other
settings. In both inputs set GUI `initial_layer_line_width=0` and global
`inner_wall_line_width=0.45`. Add object3's GUI inner-wall width override:

| case | object3 inner width | SHA256 |
| --- | --- | --- |
| control | 0.45 | `7e019a264fb7a912276bf16a3559e62c62a6f521c1397181fd01ae3a3a963e8a` |
| target | 0.6 | `b0580d24169a925fc8de7241600dd615767a41ef209a2efc3617690980fcfcc5` |

Target/control differ only in this one object-level GUI width value. Discovery
on `92238c871` found full generator-only control equality in both Native modes;
target Native brim blocks retain478+478 positive extrusion XY moves while Ares
uses321+478. The first strict residual is an earlier skirt-position move because
brim occupied footprints feed the skirt hull. Target walls legitimately change;
no claim that complete target/control streams should match. These counts are
external diagnostic evidence, not hard-coded result assertions.

Discovery evidence is external
`worker2-brim-flow-discovery-20261022T192823/DISCOVERY-REVIEW-REQUEST.md`.
No M73, timing, travel, geometry or metadata masks are permitted.

## SPEC → RED gate (current authorization ends here)

1. Commit this docs-only SPEC before adding tests or fixtures.
2. Add exactly the two frozen 3MFs and two public tests. Reuse unchanged
   `runner::native_stream_case::check` with `(false, |_, _| {})`; no new harness,
   duplicated runner, geometry callback, guard, fallback or expected-error test.
3. On the exact RED tip run both modes, CPU13 only, build jobs1, test threads1,
   PIN_OFFSET13 and seeded replay CPU13; each mode uses a fresh empty cache.
   Require Native double replay success, one strict target failure and one whole
   stream control pass. Retain public API and real CLI comparison evidence,
   CLI exit status and raw API/CLI equality. No assertion weakening.
4. Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
5. Submit RED RR and STOP. Do not write production code or run FULL without new
   authorization. Freeze tests, fixtures, SHA values and shared helper for any
   later GREEN. Coordinator owns review/merge; W2 integration-window notification
   interrupts this work immediately. Keep the previously approved skirt chain
   frozen in its separate worktree.
