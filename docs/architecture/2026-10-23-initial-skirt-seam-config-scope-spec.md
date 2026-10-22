# Initial skirt emission configuration scope SPEC

## Source-owned rewrite boundary

Rewrite the initial skirt emission configuration state of OrcaSlicer
`libslic3r/GCode.cpp` (upstream revision
`8500fcdccaa10b5099ac20d252af3a7c560046f1`), not a new Ares seam policy:

- `2533-2534`: initialize `m_config` with default object and default REGION
  configuration before layer emission.
- `4593-4594::process_layer`: apply the first layer object's OBJECT config;
  this does not eagerly apply that object's region overrides.
- `5243-5251`: emit combined skirts before the region applications associated
  with subsequent object extrusion (`5341-5342`, `6136`, `6163`).
- `4388-4451::generate_skirt`: route skirt loops into `extrude_loop`.
- `5793-5804::extrude_loop`: compute seam gap from the current `m_config`,
  convert its percentage against nozzle diameter and clip the loop end.

GUI ownership is source-cited by `PrintConfig.cpp:5515+` (`seam_gap`, a
float-or-percent region option) and `GUI/Tab.cpp:2336` (Seam gap control).

The Rust destination is the existing
`project_slice/gcode_emit/skirt.rs` clipping/emission boundary and
`project_slice/gcode_emit/motion/options` construction. Currently
`motion/options/build/region.rs:160-163` resolves seam gap using the selected
region override, and `skirt.rs:170-174` clips using that already-populated
`state.options.seam_gap`. This eager region-option state is a temporary
compatibility shell, not an independently owned skirt configuration policy.
A future GREEN must replace the incorrect initial-skirt config state with the
cited upstream default-region/first-object-object state without retaining a
legacy branch. No GREEN implementation is currently authorized.

## Included and deferred behavior

Included: initial combined-skirt emission must see the upstream default region
and first object's OBJECT configuration state, before object-region application.
Preserve existing float-or-percent resolution, nozzle selection and loop clipping
semantics; do not change the clip geometry algorithm to compensate for the wrong
configuration scope. Audit existing option construction and config lifetime before
any implementation, and report necessary direct wiring before editing.

Deferred: wall seam geometry, scarf/seam-slope algorithms, travel, cooling/time
estimation/M73, new validation, toolchanging, painted configuration behavior,
per-object skirts and later-layer config persistence until separately source-
audited. No claim is made that the whole target/control outputs should match:
object wall seams legitimately change. A strict residual outside the authorized
initial-skirt config state requires STOP rather than scope expansion.

## Frozen two-input GUI envelope

Base main `afcb6aa7c`. Seed
`tests/parity/ordered-brim-owner/far-uncombined.3mf`, SHA256
`c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6`.
Retain all meshes, transforms, placement, global settings and object metadata;
add one GUI object2 `seam_gap` override:

| case | object2 seam gap | SHA256 |
| --- | --- | --- |
| control | 10% | `d3f6e170ab2162df20575cd39ec9f289df2a899ce2d13ec10040d01666156b12` |
| target | 50% | `fe99c85d5beb6585633f12ddd518aaa37bc88b012da9b6819dd6bcfb6a9615c6` |

Only this one XML value differs between the two zipped input archives; global
seam gap remains default10%. Discovery on exact `afcb6aa7c` found strict complete
Native/CLI control equality in both modes; target first-skirt-loop endpoint
Native `G1 X134.43 Y104.59 E.01149` versus Ares
`G1 X134.313 Y104.699 E.00658`. These are external diagnostic observations, not
hard-coded test constants. Native repeats and both modes agreed. Discovery is
retained externally in
`worker2-skirt-seam-scope-discovery-20261023T034602/DISCOVERY-REVIEW-REQUEST.md`.

## SPEC → RED gate

1. Commit this docs-only SPEC before tests/inputs.
2. Add exactly two public cases and input-only 3MFs with frozen SHA. Reuse
   unchanged `runner::native_stream_case::check` with `(false, |_, _| {})`.
   No copied harness, extra geometry callback, guard, fallback, new lint
   allow/expect, assertion weakening or expected-error assertion.
3. Keep registry files within400LOC; mechanical module-list extraction with
   relative-path adjustment may be necessary because the current entry is
   already400lines. Preserve test namespaces, source paths and all declarations.
4. Exact RED tip, CPU13, build jobs1, test threads1, PIN_OFFSET13 and replayCPU13;
   both modes serial with independent fresh empty caches. Require Native double
   replay success, one genuine strict target failure and one complete control
   pass, successful API/actual CLI generation and raw API/CLI equality.
5. Compare complete ordered streams with generator-line-only normalization.
   No M73/time/travel/metadata/geometry masks or result constants.
6. Run `cargo fmt --check` and
   `cargo clippy --workspace --all-targets -- -D warnings`; submit RED RR and STOP.
   No production code or FULL before new authorization. Freeze tests, inputs
   and shared helper for future GREEN. Coordinator owns review/merge; the
   notified W2 skirt-chain integration window always has immediate priority.
