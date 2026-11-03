# APA-R2 M: standalone multipath PA mean correction — repair SPEC

Status: **REVIEW REQUEST, not implementation authorization or M closure**.
Parent: ARD-0104 / `adaptive-pa-deferred-r2-spec.md`. Upstream pin:
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Production base: `49686f943f4fab7c21d55d913b48986afed5e634`.
Test-first RED: `87bafef63b53ab80503f8340d042fbcdd8fcd97e`.
The RED stays unpushed; no production patch is included in this SPEC.

## Non-negotiable boundary

Port `GCode.cpp:6038–6066::extrude_multi_path` mean and first-child suppression
into `gcode_emit/motion/perimeter/multi_path.rs`, using the existing typed
`ExtrusionPath`/`EmitState` boundary. This is an upstream rewrite slice, not a
new Ares PA pipeline. Existing child emission is a temporary compatibility
shell: its geometry/wipe behavior stays unchanged, but its missing PA entity
context must be replaced with the cited upstream context.

Do not modify geometry, path ordering, model CSV, flavor, comments, E scaling,
scanner, PCHIP formula, rounding, formatter, comparator, reset replay or public
API. No per-tool PA cache, guessed marker protocol, legacy fallback or new
fixture. Do not widen runner field visibility.

## Authentic RED and controls

Fresh on the RED tree: `cargo clippy -p ares-cli --tests` exit0; required official
PA-off directed PASS; then required official active directed FAIL/exit100,
Native itself exit0. Full original comparator, generated-by normalization only.

Original C0 ZIP SHA `6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d`;
active ZIP SHA `01277654bc94b40af3c8acf242c952ddb2e635cc2ec2b27a9a3c4e2c54d8d7a4`.
Only enable_pressure_advance, adaptive_pressure_advance and the previously
admitted nonconstant model differ. Every other setting/non-config member stays
exact. Static PA0.02, filament ratio0.98, print ratio1, comments0, marlin retained.

First difference: normalized byte3095, line153, Native `M900 K0.064` vs Ares
`M900 K0.063`; line232 repeats it. All other ordered lines, including every XY/E
move and adjacent `G1 F7200`, are exact. M900 is the native flavor-dependent PA
command expressly admitted by the owner. Evidence/audit0:
`${TMP_ROOT}/ares-adaptive-pa-r2/M-496-RED-audit.json` and `RED-REQUEST-M.md`.

## Source-localized value chain

1. `GCode.cpp:6045–6057` resets the multipath PA flag/mean, computes
   `sum(path.mm3_per_mm * unscale(path.length())) / sum(length)` over children
   **not** marked `is_force_no_extrusion()`, without print/filament flow ratios.
   `6060–6066` sets the suppression flag after each child, independent of
   whether its PA tag was emitted.
2. `_extrude`, `6722–6742`, suppresses subsequent same-role tags; a role change
   overrides suppression. `6816–6841` emits the positive entity mean via `%g`,
   rather than the ratio-scaled `_mm3_per_mm` fallback. A single path resets
   entity context in `extrude_path`, `6103–6108`.
3. Rust `perimeter/multi_path.rs:28–41` currently emits children without setting
   the mean/suppression. `motion/path/adaptive_pa.rs:24–26` consequently uses
   `first_path.mm3_per_mm * print_flow_ratio * filament_flow_ratio`. The last
   factor is **0.98**, not1, in this exact input. Missing weighted averaging
   and the unintended fallback scaling jointly change the model's flow input.
4. `Flow.cpp:219–230` uses a float result for the rounded-rectangle area.
   `GCode.cpp:6412–6438` leaves acceleration_i0 because default_acceleration0:
   configured outer_wall_acceleration500 is not the used PA acceleration.
   Rust `motion/features.rs:97–100` likewise returns acceleration0 when the
   default is0. Confirm that gated value in the reached tag; do not substitute
   outer500.
5. `AdaptivePAProcessor.cpp:106–214,244–258` parses decimal MM3MM/ACCEL and
   scans post-cooling speed until travel/wipe/RC1. The reached feature prints
   `G1 F7200`, hence120mm/s; no overhang override in this admitted open inner
   wall. Model input is MM3MM×used_speed and acceleration_i, not E distance,
   raw requested speed, or the configured-but-gated outer acceleration.
6. `AdaptivePAInterpolator.cpp:17–79,89–120` groups the CSV by acceleration,
   evaluates flow PCHIP in each group, then acceleration PCHIP and rounds
   `std::round(prediction*1000)/1000`. Here acceleration0 clamps to model100.
   `PchipInterpolatorHelper.cpp:58–110` and Rust `adaptive_pa/pchip.rs` have
   matching endpoint slopes, harmonic interior derivatives, clamping and
   Hermite arithmetic. `adaptive_pa/model.rs:35–54` likewise rounds3dp.
7. `GCodeWriter.cpp:370–391` formats that **already rounded** value with
   defaultfloat/setprecision4 (significant digits, not four decimal places).
   Rust `pressure_advance.rs:43–99` also renders0.064/0.063 unchanged. Therefore
   changing formatting/tie-breaking to force0.064 would mask a wrong input.

## Offline numeric localization, with limits

`${TMP_ROOT}/ares-adaptive-pa-r2/offline-pa-localization.json` compiles the unmodified
upstream AdaptivePAInterpolator/Pchip sources and unmodified Rust Model/Pchip/
number modules into isolated numeric helpers. It uses the active input's exact
CSV. This executes no Native slicing or workspace tests and patches no sources.

From retained widths0.45866/0.457078 and two printed lengths approximately
0.018/4.086mm, float-area reconstruction and six-significant-digit MM3MM give:

| Source-derived input | MM3MM | flow×120 | raw PA, both helpers | rounded/written |
| --- | --- | --- | --- | --- |
| Raw length-weighted entity mean | 0.0828329 | 9.939948 | 0.06353528737772896 | 0.064 |
| First child ×0.98 fallback | 0.081485 | 9.7782 | 0.06304488184403412 | 0.063 |

Conservative width±0.000005 and endpoint-length quantization bounds keep native
raw PA in [0.0635348896,0.0635356852], versus fallback
[0.0630445158,0.0630452479]: opposite sides of the0.0635 rounding threshold.
Same-input C++/Rust raw predictions agree within1e-15, rounded results exactly,
for these inputs and20 bounded flow/acceleration controls. Supplying the same
weighted input makes both helpers produce0.064. This is **not pipeline GREEN**
and does not certify the entire interpolation domain.

These are source-derived estimates/bounds, **not measured hidden Native tags
or exact path attributes**. The official final output is measured; hidden
path count/flow/mean, ACCEL/OV/RC, suppression and model-call association still
need a read-only same-input trace before release. Comments must remain0;
instrumented diagnostics never replace original uninstrumented full streams.
The first offline audit's string-rendering comparison failure is retained as
v1-string-format-infra, corrected by comparing parsed rounded numbers; it is
not parity RED and did not change any production comparator/model.

## Minimal implementation proposed, after approval

- Reuse the existing scaled Point3 length-weighted helper at
  `motion/loop_paths/adaptive_pa.rs::mean(paths,0,scale)` for the currently
  represented extruding paths. Allow this real sibling consumer access only
  inside motion; no public API/re-export/new abstraction or copied arithmetic.
  Existing loop clipping arithmetic and behavior remain unchanged.
- At standalone entity entry initialize mean and first-child emission flag
  when adaptive PA is enabled; keep inactive motion behavior unchanged.
  Emit each child unchanged, then set suppression after that child, as upstream
  does. Retain the existing role-change override. Clear temporary entity
  context at the Rust boundary so the next existing single-path/loop owner
  starts with its own context; prove this matches upstream entry resets.
- Mean is raw path mm3_per_mm, **not** ratio-scaled extrusion E flow. Do not
  change ratio application in actual motion emission or isolated path fallback.
- The admitted C0 children are ordinary positive extruding paths. Rust's current
  ExtrusionPath lacks the independent upstream m_no_extrusion flag; mm3==0 is
  **not** an equivalent predicate. No invented flag/zero-volume heuristic.
  Forced-no-extrusion producer admission/representation is deferred, not claimed
  covered by this bounded fix. If traces show such a child here, stop/re-spec.
- No model/scanner/writer changes are justified by this RED. Stop for review if
  measured inputs contradict reconstruction or the minimum fix exceeds this
  entity context boundary.

## Verification/exit gates

1. Confirm exact source/localization and the admitted ordinary child domain via
   original-source read-only trace, preserving original full-output acceptance.
2. After approved patch, fresh clippy/fmt, required official PA-off PASS, then
   active full-file PASS on the **same** active input/member hashes/model,
   without expected-value edits. Enumerate all changed commands and prove no
   new non-PA/movement/E difference. Preserve original RED binary and receipts.
3. Verify first-child/same-role suppression, role-change override and context
   teardown from traces; no unit tests added after production code. Existing
   test-first controls must remain unchanged. Additional E2E controls, if
   needed, are committed before code and require admitted upstream producers.
4. Independent BASE/CAND artifacts/caches/binaries and standard parity/workspace/
   strict/WASM gates follow owner queue/load/fixed-marker protocol. Latest-main
   advances VOID gates; no integration/push without exact-hash approval.

This fixes only the bounded standalone multipath slice. Mixed-role/sloped M
residuals still need their own authentic evidence. A/R, calibration public-mode
admission, Type1 tower reset, actual Type2 activation, equalizer algorithm,
geometry and worker3/other producers are deferred. Parent `2863ef3ea` and M
remain OPEN until their separately defined criteria are satisfied.
