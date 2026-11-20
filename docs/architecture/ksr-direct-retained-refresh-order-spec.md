# KSR direct retained refresh order — conditional rewrite slice

## Scope

The source boundary is retained-preset construction on the direct catalog-bearing project seam, not a new timelapse or timing subsystem.

## Exact observed branch

Original input SHA remains `698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`. It names `Bambu Lab X2D 0.4 nozzle`, contains `printer_structure=corexy`, two 0.4 nozzles, traditional `timelapse_type=0`, and no `farthest_point_timelapse`. Effective output has `enable_prime_tower=0`. Upstream system resource `resources/profiles/BBL/machine/Bambu Lab X2D 0.4 nozzle.json:13` sets `farthest_point_timelapse=1`.

Thus Native uses the non-I3, traditional, no-wipe-tower farthest-point inline hook. It is not smooth mode, not the ordinary I3 post-perimeter/pre-infill insert, and not a wipe-tower toolchange insert. The regular dual-head traditional layer-end fallback only applies if no inline snapshot was inserted.

All source citations below are actual lines in the complete `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` source. Earlier citations around `GCode.cpp:5455–5460,5527–5546,6480–6487,6516–6550` refer to older line layouts; their owning functions/branches, not those obsolete absolute ranges, identify the port boundary.

## First executable divergence, line by line

The following compares Native output with direct catalog slicing before the load-order fix. The browser uses that same direct core seam.

| Native line(s) | Ares line(s) | Meaning |
|---|---|---|
| 1653–1654: `G1 X169.239 Y82.292 E2.72383`; `G1 X169.239 Y144.692 E2.32416` | 1652–1653: same two extrusion commands | Same incoming path and endpoint. |
| 1655–1658: template heading, `SKIPPABLE_START`, `SKIPTYPE: timelapse` | absent here | Native emits the inline template after the endpoint. |
| 1659: `M622.1 S1 ; for prev firware, default turned on` | absent here | First executable difference; begins the template's firmware conditional setup. |
| 1660,1662,1664–1669 | absent here | Template whitespace, retained in byte comparisons. |
| 1661: `M1002 judge_flag timelapse_record_flag` | absent here | Firmware flag selection. |
| 1663: `M622 J1` | absent here | Begin conditional snapshot region. |
| 1670: `M9711 M0 E1 Z0.6 S11 C10 O0 T3000` | absent here | Traditional (`M0`), photo physical head 1, inline capture without X/Y relocation. |
| 1671–1676 | absent here | Retained template whitespace. |
| 1677–1679: `M623`, `SKIPPABLE_END`, blank | absent here | End snapshot block. |
| 1680–1681: `G1 X136.839 Y144.692 E1.20678`; `G1 X136.839 Y100.632 E1.64107` | 1654–1655: exactly these commands | These commands are not extra Ares geometry; Ares simply continues before taking the snapshot. |
| 1682: `M204 S6000`; subsequent retract/travel | 1656: `M204 S6000`; subsequent retract/travel | Native's inline generator has cleared writer position; Ares still uses the clear-position spiral lift (`G17`/`G3 Z.6` at 1660–1661), so later travel differs as a consequence. |

Ares does not omit all snapshots: its first is delayed to line 3090, `M9711 M0 E1 X133 Y193 Z0.6 S11 C10 O0 T3000`, at the ordinary layer-end path. That is an added safe-position relocation relative to Native's first inline snapshot, not evidence for adding another hook call.

## Upstream ownership and existing Rust match

- `GCode.cpp:6711–6713` enables farthest-point only for flag=true, traditional, non-I3; `:6717–6723` computes ordinary traditional/tower conditions.
- `GCode.cpp:9974–9996` owns the endpoint inline hook; it calls `generate_timelapse_gcode(..., true)` after the photo head reaches the farthest endpoint. Fitted-linear emission calls it at `:10148`. Rust already calls `layers::farthest::check_and_insert` from `motion/path/constant.rs:106–112` (and after arcs at :143–149).
- Rust `layers/farthest.rs:37–40` folds the same enable predicates; `:76–94` applies the photo-head, already-inserted and endpoint-distance predicates. Its flag comes from finalized typed printer config, not the browser.
- `GCode.cpp:6494–6541` `generate_timelapse_gcode` renders and adopts last Z, and clears writer position at `:6509`; `:6543–6600` `timelapse_gcode_config` sets inline/safe-position context. Rust `timelapse.rs:55–95,168–207` implements this path. No hook or template-render change is indicated by the observed first difference.
- `GCode.cpp:8193–8227` suppresses ordinary layer-end insertion after an inline snapshot, otherwise emits the traditional fallback. Rust `layers/layer_end.rs` and `motion.rs::end_layer_for_timelapse` consume the same inserted flag.

## Owning load-order deviation

`OrcaSlicer.cpp:3164–3200` constructs retained printer/process presets from collection defaults, project membership and current system resolution; missing saved options receive the current system value. X2D therefore supplies the absent timelapse flag as 1.

Direct catalog loading previously called `project::load_project_with_preparation` before `refresh_direct`. `project/load.rs:86–90,191–193` materializes retained defaults first; printer option `gcode_source.rs:78` supplies 0. `profiles/project_refresh/preset.rs:50–57` then preserves this now-present key when `refresh_unlisted=false`. The direct loader has erased original membership before catalog refresh. The profile-owned loader already avoids this by using `load_project_for_profiles` and only applying remaining collection defaults after refresh (`profiles/project_refresh.rs:172–194`).

This is the same ordering contract documented by `retained-project-preset-membership.md` and explicitly deferred for original KSR in `2026-11-13-farthest-point-snapshot-wall-paths.md:81–83`. Do not work around it by forcing the flag, changing its default, moving snapshots, or adding KSR-specific conditionals.

## Conditional implementation boundary

Defer retained default construction for catalog-bearing direct BBL loads until after existing system refresh, then fill remaining defaults and apply caller overrides. Preserve the non-BBL/no-catalog and explicitly selected-profile boundaries, preparation snapshot, genuine slicing warnings, connection credential exclusion and missing-bundle error. An unresolved filament keeps its slot without a new `PrintWarning`: `OrcaSlicer.cpp:2805–2807` only logs and continues, and `PrintBase.hpp:66–75` has no such notification. Reuse existing upstream-owned constructors; core remains I/O-free.

Included: raw membership ordering at the existing direct project load seam. Deferred: new machine-selection semantics, new timelapse algorithms, geometry ordering and all processor/TimeMachine behavior. Existing typed/raw Ares scaffold remains a temporary compatibility shell around retained Native preset construction.

## Failing-first and verification contract

Retain the original KSR complete ordered-byte witness and comparator unchanged. The full comparator currently first differs at byte 77 in the estimate header; if that header remains different after this fix, its literal first difference cannot move. Separately record the first executable difference to verify the specific timelapse block moves into place; this diagnostic does not normalize/replace the complete comparator. If estimates still differ afterward, retain the literal first difference and diagnose the timing discrepancy separately; do not change this comparator.

Before implementation, capture the failing baseline with a new public E2E under the existing Native-backed KSR producer. Native exports the unchanged project's effective settings. The public direct catalog slice uses no overrides. Its first snapshot's inline-versus-relocated form observes effective `farthest_point_timelapse` (this key is intentionally excluded from exported CONFIG_BLOCK and placeholder values); compare it with Native's exported option. For `enable_pre_heating`, `group_algo_with_time`, `bottom_color_penetration_layers`, `top_color_penetration_layers`, and `extruder_clearance_dist_to_rod`, compare exported G-code config values with Native's exported settings. Neither the KSR fixture nor the full-output witness changes. Keep original project SHA and publish native settings, API G-code, catalog identity, and per-key comparison artifacts. Base must fail and the fix must pass. After implementation: unchanged inventory in official and seeded full frozen environments must have P→F=0, including direct/selected/non-BBL controls. No fixture metadata or retained assertions may change. Native supplier exclusions remain explicit UNVERIFIED.
