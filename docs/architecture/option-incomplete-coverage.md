# INCOMPLETE coverage: classification and activation audit

## Inventory and non-negotiable decisions

The seeded option-coverage report contains225 INCOMPLETE domains, not225
mismatches.

| Reason | Count |
|---|---:|
| Implicit upper endpoint/full-range seed unverified |151|
| Both implicit endpoints/full-range seed unverified |39|
| Authored interior probes; schema endpoints rejected by native validation |13|
| Missing required legal cases/combinations |12|
| Implicit lower endpoint/full-range seed unverified |4|
| Single-nozzle context only |2|
| Named modes but unbounded integer endpoints |1|
| Unsupported numeric bound expression |1|
| Implicit FLT_MAX ceiling |1|
| Timelapse native input rejection |1|

Completeness and feature activation are independent. More active probes do
not discharge unverified endpoints. Never replace an implicit maximum with
an arbitrary finite number; preserve comparison, requirements, rejection
records and INCOMPLETE status. Tower/AdaptivePA retain their other owners.

## Rejected skirt candidate

A proposed skirt witness initially required one loop. Both references
rejected that expectation.
Saved3MF readback shows the existing fixture already has two loops, two
layers and a disabled draft shield. This is not missing activation or a
production regression: the new expectation was unnecessary. Withdraw that
uncommitted test; make no skirt activation or production changes. Retain
the record rather than fabricate a GREEN by forcing an arbitrary loop count.

## Actual candidate: filament ironing speed

`PrintConfig.cpp:3529–3539` declares nullable filament speed, minimum1.
`Fill/Fill.cpp::Layer::make_ironing` (1575–1604) gates all ironing on
`ironing_type` and chooses the non-nil filament override before process speed.
Rust ownership is `project_slice/fill_entities/ironing.rs` plus region-resolved
speed assembly in `gcode_emit/motion/options/build/fields/assemble.rs:97–99`.
The existing Ares emission layer is only the implementation boundary for these
upstream concepts, not a new independently designed slicing pipeline.

The option activation table already enables process `ironing_speed` and
filament flow/spacing/inset, but omits `filament_ironing_speed`. Add only the
missing `ironing_type=top` fixture dependency after public RED. The ironing
fan candidate is a separate slice.

## Plan and acceptance

1. Add a public saved3MF test over every existing filament-speed case. Verify
   applied values, `ironing_type=top`, actual positive relative-E XY ironing
   extrusion and strict whole-output comparison. No private emitter-only test.
2. Capture both references before changing activation. A missing enabler
   is fixture RED, not yet an Ares production output mismatch.
3. Add the enabler and repeat both references. Preserve all cases and the
   implicit-upper-bound INCOMPLETE verdict. Investigate any actual divergence
   against the cited owner before changing production.
4. Run full workspace Nextest, Clippy, rustfmt and core/adapter WASM build.

RED: both references exit100 because saved `ironing_type` is `no ironing`, not
`top`. The one-line activation fix follows this evidence; production is
unchanged at that checkpoint. Every existing case remains in the public test and matrix.

## Production RED and exact rewrite boundary

The attempted GREEN instead
exposed deterministic DIVERGENT output with both references. All2 matrix
cases were strictly compared with0 rejection; each disagreed across3 native
runs. The minimum case passed saved-value/active-extrusion witnesses but
emitted F60 in Ares versus native F900, with M73 R24 versus R9.

`Fill/Fill.cpp::Layer::make_ironing` stores filament-specific speed in its
ironing parameters, but the generated erIroning paths at1727–1730 carry
flow/width/height, not that speed. The emission owner is
`GCode.cpp::_extrude`,6538–6539: it selects region `ironing_speed` from
`m_config`, not `filament_ironing_speed`. The motion options assembly above
incorrectly promoted the filament parameter to emission speed.

Port that exact selection: use resolved region process `ironing_speed` in
`gcode_emit/motion/options/build/fields/assemble.rs`. Retain the nullable
filament parameter in the region model (including its equality/grouping
identity); do not remove or reject it and do not invent intended semantics
that the native2.4.2 emitter does not execute. Geometry, fan lifecycle,
multitool policy and global domain-bound completion are outside this fix.
The existing public complete-output RED is the production regression test.

## Final candidate verification

Official and seeded each pass2 public active-path comparisons and21 matrix
comparisons across8 domains. The matrix deliberately exits100:6 bounded
PASS and2 INCOMPLETE speed domains,21 generated/compared,0 rejection,
0 DIVERGENT. Both incomplete domains still lack verified upper endpoints;
this is not a claim that the225-domain inventory is closed.

Workspace Nextest, all-target Clippy, fmt check and core/adapter WASM
compilation pass. WASM is compile-only evidence. All changed Rust files
remain below400LOC. No skirt change, comparator relaxation, deleted existing
assertion, unsupported guard or debug probe is included.
