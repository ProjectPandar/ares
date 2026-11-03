# Object-wide AutoBrim adhesion rewrite SPEC

Status: **COORDINATOR-APPROVED SPEC**. Original discovery was external/read-only.
Explicit dependency: approved ears-EFC tip
`0688f83a6ff33c6de6e061b5bc93cf11f6c89ead`, including AutoEFC `14d90d03f`
and exact-group `fb70752b9`. SPEC and public RED stack on that tip.
No rebase or FULL before the W2 integration window.
Source baseline main `231bd11cc`; runtime is explicitly **not an exact-main binary**:
existing approved `0688f83a6` CLI, SHA256 in provenance.txt, including approved
exact-group/AutoEFC/EarsEFC fixes. This is a dependency-stack witness that removes
the old group-assignment confound, not a claim of new public RED on main.

## Upstream boundary / exact Rust destination

Upstream checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
- `Brim.cpp:120-148::getadhesionCoeff`: external-perimeter extruders of first-layer
  regions **with actual extrusions**, then all object volumes in model order;
  last matching volume with an extruder-material map entry assigns adhesion.
- `Brim.cpp:477`: compute object adhesion before the group's width call at `501`.
- `Brim.cpp:260-305::configBrimWidthByVolumeGroups`: object adhesion multiplies
  the group moment/thermal width term.
- `Model.cpp:3258-3280::getThermalLength`: group-scoped minimum thermal reference;
  it must NOT accidentally become object-wide along with adhesion.
- `MaterialType.cpp:134-168`: existing adhesion/thermal table vocabulary.

Main `crates/ares-core/src/project_slice/gcode_emit/brim/auto.rs::width_mm` currently
filters material names by `WidthContext::volume_ids`, and `material_terms` returns
both adhesion and thermal from this group-only list. It explicitly defers object
adhesion. `brim/groups.rs::volume_materials` supplies ordered object volume data.

Prospective rewrite boundary: object material-selection preparation in
`gcode_emit/brim.rs` / `brim/groups.rs`, and `auto.rs::WidthContext/width_mm`.
Compute one object adhesion from actual first-layer external-perimeter extrusion
eligibility and ordered source volumes; pass it to all groups. Keep thermal
selection group-scoped. Do not simply take the max coefficient, first filament,
last group, or all configured/unused filament slots. This replaces the temporary
group-only adhesion carrier with the cited upstream concept, not a new pipeline.

Included if authorized: existing material tables; first-layer actual-region
eligibility; model-volume order; object-wide coefficient reused across groups;
existing per-group thermal length untouched. Adjacent geometry, second moment,
width quantization, max-speed, EFC, routing, skirts, material/footer serialization,
seams, fill, ironing and all W1/W3/W4/W5/W6 work deferred. No guards/fallbacks.

## Inputs and source provenance

Original input generator copied from the earlier external material discovery;
source is tracked `tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf`,
SHA256 `e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5`.
Generator adds a separate second model-part component, same mesh, translated30mm,
extruder2; source object/first volume extruder1. Both parts extrude first layer,
by-layer, classic, support/tower disabled, no skirts, combine_brims0, EFC flag0.
Filament slot arrays extended uniformly; only second-slot PLA/PETG is varied.
This is reachable via adding a part, assigning filaments and setting brim type.
The generator's old absolute source path is retained transparently; verify source
SHA against tracked fixture before replay. `inputs/inputs.json` hashes all4files.

Factorial: `auto_brim|no_brim` × `PLA/PLA|PLA/PETG`. All4 frozen input-only archives
are referenced by run.sh and verify-motion.js. No outputs/probes enter Git.

## Read-only Native-first results

All8 official/seeded Native runs rc0, real CLI4/4 rc0, allCPU13 and seededCPU13.
Only generator line normalized; no M73, motion, temperature or metadata mask.

| case | Native positive Brim XY | Ares | strict whole output, both modes |
|---|---:|---:|---|
| Auto PLA/PLA |894|894|equal|
| NoBrim PLA/PLA |0|0|equal|
| Auto PLA/PETG |2596|1742|different|
| NoBrim PLA/PETG |0|0|equal|

Native active material change grows actual Brim extrusion vs PLA/PLA; this is not
an inactive coefficient probe. Target first differing positive-Brim motion1295:
Native `G1 X129.616 Y119.995 E.30704`, Ares `G1 X124.336 Y119.993 E.30693`.
That diagnostic role extraction proves geometry differs; **it is not an acceptance
comparison or a replacement for strict full-output parity**.

## Current limitation — do not overclaim RED qualification

Strict first full-output difference is byte723, M73 R134 vs R133. There is real
geometry difference behind it, so it is not an M73-only case, but this does NOT
meet a requirement that the strict *first* difference be non-M73. No public API
harness has been added, no public RED result claimed, and no GREEN is requested.

Uniform GUI skirt_speed200 and initial_layer_speed100 probes preserve geometry
mismatch and all3strict-equal controls, but still lead with M73. A uniform GUI
Z-scale37.5 (75mm height) probe gives Native1074/Ares537, all3controls equal, yet
first strict difference is a prior M73 insertion at1113. These external probe
inputs are NOT proposed as final fixtures; no further timing tuning or masking.

Coordinator explicitly accepted this M73-leading strict failure because the
independent Native/Ares Brim geometry evidence establishes its cause. Preserve
M73 in complete ordered comparisons: no mask or timing-only acceptance.
The authorized public RED must use shared NativeCase
for Native→API+actualCLI and rawAPI/CLI equality; no copied EFC/general harness,
new ignore/allow/expect, constants encoding results or M73 masking.

## Evidence / next decision

Root contains run.sh, generate-inputs.js, inputs.json, per-mode Native logs/rc,
actualCLI logs/rc, summary.json, verify-motion.js, verified-motion.json,
verification.txt, and provenance.txt. All Native output stays external.
Failed python3 launch (unavailable executable) produced no runs; subsequent
replayable Node generator succeeded. No environment installation was performed.

Coordinator approved committing this docs-only SPEC, then frozen-input NativeCase
RED with API + actual CLI strict complete output and Native-first Brim change.
Report RED before any production changes. Prospective GREEN remains limited to
object adhesion preparation and WidthContext/width_mm; thermal stays group scoped.
Any strict residual requires STOP rather than broadening the boundary.
All evidence paths in this document are relative to the external discovery root:
`${COORD_ROOT}/evidence/worker2-material-rediscovery-20261020T224150/`.
