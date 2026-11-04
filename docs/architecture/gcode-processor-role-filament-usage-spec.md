# GCodeProcessor role filament usage: source boundary and RED design

Upstream source boundary:
`OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.{hpp,cpp}` and the data
consumption in `OrcaSlicer/src/slic3r/GUI/GCodeViewer.cpp`.

## Current main audit

The deferred table in
`docs/superpowers/specs/2026-10-06-filament-usage-stats.md:57-58` is historical.
The two rows now have different implementation status:

| Upstream field | Current main | Decision |
| --- | --- | --- |
| `volumes_per_color_change` | Present in `ares-core/src/gcode_processor.rs:98`; accumulated/flushed in `project_slice/gcode_emit/processor/moves/extrusion.rs:25-34,68-74`; transferred into final statistics in `processor.rs:225-228` | Already ported; retain unchanged |
| `used_filaments_per_role` | Absent from core statistics and accounting, vgcode legend data, and WASM/JS result consumer | Port in this slice |

Paths in the table are relative to `crates/`. Color-volume consumption also
exists in `ares-vgcode/src/viewer/color_print.rs:160-161`, with Native raw-volume
comparison in `ares-core/tests/processed_color_change/native.rs:86-87,230-233`
and browser coverage in `web/e2e/colorChange.spec.ts`. That is stronger evidence
than merely finding a field declaration. Existing ColorPrint partial usage,
custom-time intervals, marker correction and associated oracle distinctions
remain in place; this slice does not reimplement them.

The current FeatureType rows in `web/app/src/preview/Legend.tsx:118-135` contain
role labels and visibility buttons, without role filament length/weight.
`UsedFilaments` (`processor/used_filaments.rs`) owns categorized volume totals;
`MoveRecorder` owns the already-ported color cache. This split is an existing
temporary compatibility shell around upstream GCodeProcessor, not a new
Ares-owned statistics pipeline.

## Included upstream behavior

All C++ line citations below refer to the checked-out OrcaSlicer source.

| Source | Required behavior |
| --- | --- |
| `GCodeProcessor.hpp:46-78,583-619` | Sparse map from `ExtrusionRole` to a pair of doubles: filament length in **meters**, mass in **grams**; reset clears it |
| `GCodeProcessor.cpp:1525-1547,3895-3911` | Support, model and wipe-tower extrusion all add the same per-move float volume to the double role cache |
| `GCodeProcessor.cpp:1621-1639` | Flush nonzero role cache using the current filament diameter/density and current role; accumulate into that role's pair; reset cache |
| `GCodeProcessor.cpp:5720-5724,3121` | Flush old role before every `set_extrusion_role`, including repeated same-role tags, then assign new role |
| `GCodeProcessor.cpp:5937-5950` | ToolChange flushes model/support/total caches, **not role cache**; ColorChange flushes only color volume here |
| `GCodeProcessor.cpp:1641-1649,2709,6031-6037` | Finalize pending caches, then copy `filaments_per_role` into public `used_filaments_per_role` |
| `GCodeProcessor.cpp:1606-1619,3955-3968,4306-4324` | Flushing updates flush/total volume; it does not add to role cache; virtual flush likewise contributes no role usage |
| `GCodeViewer.cpp:3432-3441,3652-3655,3881-3882` | FeatureType looks up the core extrusion role, defaults missing role to zero, and displays its length/weight |
| `GCodeViewer.cpp:128-151` | Metric weight formatting uses two decimals and g/kg/t thresholds; metric length uses `%.2fm` at line 3653 |

The conversion order is significant. Preserve upstream's double expressions:

```text
s = PI * sqr(0.5 * filament_diameter_f32)
meters = role_cache / s * 0.001
grams = role_cache * filament_density_f32 * 0.001
```

Diameter and density begin as processor f32 metadata and promote to f64 for
these expressions. Volume begins as the existing processor's per-move f32
extruded volume and accumulates in f64; do not reconstruct it from rounded
footer values or `MoveVertex.width * height`. Conversion occurs at cache flush,
not once after summing all volume across filaments. In particular, a role cache
can span a ToolChange and is then converted using the filament active at its
next role/final flush. Port this defined upstream behavior even when materials
have different properties; do not silently add an extra tool-change flush.

## Rust destination and boundaries

1. `ares-core/src/gcode_processor.rs::PrintEstimatedStatistics`: add
   `used_filaments_per_role` as a sparse `BTreeMap<ExtrusionRole, (f64, f64)>`.
   Add the minimal role ordering/serialization derives needed by the map;
   serialize role names, never reinterpret core enum ordinal as libvgcode role.
2. `ares-core/src/project_slice/gcode_emit/processor/used_filaments.rs`: keep
   one double role cache and active core role. Before a supported role tag,
   flush using the active filament, then replace the role. Add volume alongside
   existing ordinary extrusion accounting; finalize pending role volume before
   returning statistics. Use existing settings/processor metadata semantics
   for diameter and density. Keep `estimate.rs` wiring minimal (it is already
   over 400 LOC); any substantial new logic belongs in the accounting module.
   Existing final statistics replacement in `processor.rs` must retain this map.
3. `ares-vgcode`: a small rendering-neutral feature-usage projection uses the
   existing explicit core-to-vgcode role mapping in `convert.rs:116-139` and
   the viewer's used roles. Rows carry role id plus meters/grams, with zero for
   a missing core entry, matching the Native lookup. No G-code reparsing here.
4. `ares-wasm/src/moves.rs`, `web/packages/ares-wasm-js/src/{worker,toolpaths}.ts`:
   carry engine-produced feature usage to the existing toolpath data. The
   public statistics JSON also includes the core map through its current
   serialization path. The adapter performs no independent accounting.
5. `web/app/src/preview/Legend.tsx`: add metric length and weight to existing
   FeatureType role rows, matching the source formatting above. Preserve role
   visibility behavior, option rows and ColorPrint legend behavior.

Core/vgcode remain platform-neutral and contain no file I/O or debugger code.
No new crate, statistics service, parser fallback or feature flag is needed.

Deferred: feature-time/percentage columns, imperial-unit UI selection, broader
producer-specific tag parsing, changes to color-volume/custom-time semantics,
and unrelated processor motion precision changes. Existing supported role tags
and motion accounting are the input boundary for this slice. No G-code command,
M73 value, footer total, slicing path or role selection should change.

## Native oracle capability and RED design

The current `tools/chain-probe/procgcode.patch:64-97` exports raw color volumes
and per-filament totals but **does not export the role map**. A footer total
cannot distinguish correct role attribution from moving all usage to one role;
it is a control assertion, not a substitute role oracle. No RED is claimed yet.

Use the existing raw procgcode binary, without rebuilding OrcaSlicer:

```text
/nix/store/1bq61dyqxb8yjdsaaq697ii6hbhb5cxm-orca-slicer-procgcode-2.4.2/bin/orca-slicer
ELF: bin/.orca-slicer-wrapped
ELF SHA256: 59f9ea7e16165df6979a243de7d6acd49d8cf523ac5de6d3fec08d6e25c7896e
```

Read-only `nm -C` confirms `UsedFilaments::process_role_cache` is present
(`0x908a80`), as is `process_file` (`0x913ec0`). The RED harness will run
`nix shell nixpkgs#gdb -c gdb ...` on this existing ELF, preserve its runtime
environment, and read the complete Native role map after final cache processing.
Before freezing an oracle, verify structure/member offsets against source and
the executable's disassembly; symbol presence alone does not validate offsets.
Export every role key and both raw f64 bit patterns, plus Native color/totals
JSON and logs. The map snapshot must come from Native memory, not a separate
Rust/Python implementation of the formula. Do not mutate Native state or use
the corrected-options diagnostic binary as a replacement for raw Native.
If reliable extraction cannot be established, report an oracle blocker and
stop before production changes; no diagnostic Native rebuild is allowed.

Proposed fixtures and failure signals (RED tests precede implementation):

| Input | Native observable / failure caught |
| --- | --- |
| Real small project with outer/inner walls and infill | Nonempty complete Native role map; baseline serialized Ares map is absent, so the current public API test compiles and fails on missing data |
| Same role repeated, role revisited, last role at EOF | Complete key set and pair bits catch cache reset, missing accumulation and missing final flush |
| Support/interface/transition and tower extrusion | Those volumes contribute to their roles; ordinary role results must not absorb them |
| Two filaments with unequal diameter/density, explicit role tags before/after switch, plus same-role span across switch | Verify flush-time conversion and Native's no-role-flush ToolChange behavior; avoid relying on identical materials to hide errors |
| ColorChange/pause/retract/travel/wipe, E-only flush and VG1 virtual flush | Preserve existing color vector; no role volume for non-extrusion/flush paths |

Fixtures should enter through the public slice/processed-G-code APIs and retain
the actual input bytes. Minimal G-code boundary fixtures complement a real
project export when a precise cache sequence cannot be reliably requested from
geometry alone. No manually calculated constant is the reference. Preserve
fresh Native execution provenance, input SHA, role-map artifact, complete Ares
statistics JSON, Native log and comparison result. Compare complete role maps,
including absence/presence and both f64 values, without arbitrary tolerance.

The RED first failure is a processor-result difference, not an expected G-code
byte difference: this field does not alter G-code. For the real project control,
also compare full Native/Ares G-code bytes with only generated-by normalization
and existing footer observables. If an unrelated slicing difference prevents
that control, choose a parity-clean fixture rather than masking it. GREEN must
leave all original G-code/footer bytes unchanged while making the Native role
result equal. Add WASM projection and browser FeatureType assertions against
the same Native-derived usage records, including visibility toggles and metric
display, in the RED before their production changes.

## Acceptance sequence

Establish the live Native map extraction, freeze fixtures and commit RED
alone. Then implement the narrowly scoped GREEN. Run the focused Native result
comparisons and existing color-change coverage, paired seeded/official
controls (candidate failures a subset of base, all transitions explained),
workspace nextest, strict clippy, fmt and wasm32 checks. Retain repeatable
artifacts.
