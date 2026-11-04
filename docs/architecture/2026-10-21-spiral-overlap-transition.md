# ARD: overlapping spiral entry and final-layer transitions

Status: Accepted SPEC; formal RED, then a separate GREEN.

## Upstream owner and Rust destination

OrcaSlicer2.4.2 `src/libslic3r/GCode/SpiralVase.cpp::SpiralVase::process_layer`:

-122–123 define transition_in and transition_out independently;
-153–166 use if(transition_in) / else if(transition_out) for extrusion:
 entry E ramp wins and does not also produce a finishing extrusion copy;
-206–209 copy passthrough lines whenever transition_out is true, even
 if transition_in is simultaneously true;
-215 returns new_gcode + transition_gcode.

Caller `GCode.cpp:3704–3705` passes whether this is the final logical
layer. These are filter state/output semantics, NOT physical scheduling.

Rust destination:
`crates/ares-core/src/gcode_spiral_vase/project.rs::ProjectSpiralVaseRunState::process_layer`.
Current line72 globally excludes transition_in from transition_out;
extrusion branches116/121 are independent if statements. This is an
incomplete compatibility shell around the cited source, not an Ares
policy to preserve. Replace the global exclusion with independent final
state and make extrusion branches mutually exclusive in Native order.
Reuse existing passthrough handling and concatenation. Merely deleting
!transition_in while retaining two extrusion ifs would introduce wrong
finishing XY/E copies and is NOT the intended rewrite.

No new height/layer-count/shape condition or fallback. No header or
processor patch. No new Ares pipeline or public API.

## Native-first actual exported sole-key inputs

Tiny closed octagonal cylinder radius6mm/height0.4mm,18vertices/32triangles.
Two real layer inputs, printZ0.2/0.4. GUI Spiral vase option is
PrintConfig.cpp5829+, spiral_mode coBool. Both projects retain Classic,
one wall,0% infill/no top or bottom shells,0.2mm layers, aligned,
staggerOFF/scarfNONE/wipeOFF/no brim/skirt/relativeE. Both layer-change
retraction settings0, including filament override, preventing companion
normalization changes. No malformed scalar arrays or post-export edits.

| Actual official CLI export | SHA256 |
|---|---|
| control spiral0 | 7a476876c3242aa7936de75e685b2c3292a32c678bf53f994b36d6c1e2308b77 |
| positive spiral1 | d35033bde4712fd6befa3b7341c172901e654608be904cab43757cdcf6890621 |

Only changed ZIP payload Metadata/project_settings.config; only changed
JSON key spiral_mode0→1. Frozen files will be INPUTS only, not expected
G-code snapshots. Fresh official twice and seeded all agree per case:
control completePASS, positive672 aligned differences. First strict
non-M73 mismatch, line3:

```
Native: ; total layer number: 3
Ares:   ; total layer number: 2
```

## Why the header discrepancy is NOT a third physical layer

Read-only proof on exact89560: both filters receive exactly two layer
chunks. First disabled/nonfinal; second enabled/first-enabled/FINAL.
Incoming motion streams identical. Normal transformed chunk identical
except Rust's existing internal fan marker representation. Before return,
Native has a259byte NON-EXTRUDING transition chunk; Rust has none. This
chunk repeats LAYER_CHANGE/Z0.4/HEIGHT0.2, G92E0, width/feedrate/object
labels, but no extruding XY motion because transition_in wins the
extrusion branch. Native footer still reports two actual layers.

Existing `gcode_emit/orchestration.rs:350–356` counts emitted
LAYER_CHANGE chunks and calls header::finalize_layer_count. Thus line3
3vs2 is a downstream symptom of missing final chunk; fix the owning
SpiralVase semantics, NEVER force header3 or add a physical layer.
M73 and all other final-output bytes remain in strict comparison.

Source evidence: final valid debugger runs preserve respective complete CLI
outputs. No inferior calls or memory/register writes. Internal fan-marker mapping is diagnostic only,
not final-output acceptance filtering. Failed optimized-local/empty-string
reader attempts are not counted as successful proof. Exploratory absoluteE
case was rejected by Native due inherited G92E0 and is not a parity case.

## Included / deferred

Included in GREEN: independent entry/final state, Native
if/else-if extrusion priority, existing transition passthrough copying
and new+transition concatenation. Preserve arithmetic, normal moves,
physical layer schedule, marker ownership and already-fixed placement
gate. Stop/report any residual rather than expand scope. No new helper,
public API, dependency, I/O or Rust file over400LOC.

Deferred/excluded: brim/skirt; painted/MMS; medial_axis and
interlocking; combine-solid clearance. Also no
GCodeProcessor seam/scarf cache/flags or layer interpretation change,
header special case, physical scheduling/height/epsilon patch, flow
numeric rewrite, spiral smoothing/resampling, travel/retract/wipe change,
absoluteE G-code cleanup or further placement work.

## Formal RED plan

Freeze the two actual exports with whole/per-ZIP hashes and sole-key
manifest. Tests directly call main's native_stream_case::check with
Native double replay then full validated API and CLI comparisons. No
copied runner, manual normalization/replay/JSON, private seam, extra
Native content assertion, borrowed gate, ignored or expected-failure test.
Only the existing canonical generator identity/date normalization;
no M73, header, comments, E, timing, motion or trailing-line mask.

ExactRED official/seeded must show actual positiveFAIL and controlPASS,
with stable repeated Native streams, before modifying production. Future
GREEN requires both
complete modesPASS and preservation of extrusion priority/no extra XY
copy. Targeted neighbors/static/WASM and full-suite checks are separate
steps. Discovery CLI/debugger proof is not formal public-API RED/GREEN
evidence.
