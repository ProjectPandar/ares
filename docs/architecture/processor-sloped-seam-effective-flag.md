# GCodeProcessor sloped-loop seam lifecycle and effective scarf flag

## Upstream boundary

This is a Rust rewrite slice of OrcaSlicer libslic3r, not an Ares time model.

1. `PrintApply.cpp:1148–1158` owns the complete derived flag expression:
   preserve explicit incoming `has_scarf_joint_seam=true`, otherwise OR across
   every model object, using:
   - the object's effective `seam_slope_type` (explicit object value or inherited
     full-config value) != None;
   - ANY volume with an explicit `seam_slope_type` != None;
   - ANY layer-config range with an explicit `seam_slope_type` != None.
   The three sources are independent OR terms. Volume or range None does not
   erase an object's true term. An object's explicit None overrides its inherited
   default for the OBJECT term, but cannot suppress a true volume/range term.
   Do not replace this expression with global-default-only or final-region-only
   scanning. Do not filter source volumes by kind, geometry, visibility, emitted
   paths or whether a corresponding override wins another merge.
2. `GCode/GCodeProcessor.cpp:2141–2147,:2471–2477::apply_config` owns
   `detect_layer_based_on_tag = spiral_mode OR has_scarf_joint_seam`.
3. `GCodeProcessor.cpp:4199–4204::process_G1` refreshes an active external-wall
   detector's remembered first vertex when the PREVIOUS stored vertex has higher
   Z than that first vertex. The guard above must be true. This is not assignment
   of the current move endpoint, a changed closure tolerance or a new seam policy.
4. The existing closure (`:4210–4221`) and cache eligibility
   (`TimeMachine::calculate_time:484–488,:574`) then associate a Seam vertex with
   the block. Export cache lookup (`:1241–1252,:1458–1471`) remains a consumer.

The effective flag is a necessary dependency of this SAME slice,
not a separately deferred prerequisite: without it this witness cannot turn green.

## Exact Rust destination and single owner

- The ONLY PrintApply any_of owner is the existing
  `print_apply/apply_scarf_joint_seam_state.rs::staged_apply_scarf_joint_seam`.
  Widen only the required module/type/function visibility to pub(crate); preserve
  its formula and existing unit tests. The
  `project/effective_config/scarf_joint_seam.rs` module is only a data adapter:
  construct `StagedApplyScarfJointSeamObject` from original typed object defaults,
  volume and layer-range option sources, call that owner and apply its set record.
  Preserve an incoming true when no set record is returned. No second formula.
- `ProjectObject::region_overrides`, `volumes`, and `layer_config_ranges` preserve
  the three source paths. Reuse typed override/inheritance access, without resolving
  or cloning complete per-volume configs merely to inspect this one enum.
- Feed the derived flag through existing effective-config/slicing handoffs to
  `gcode_emit/processor.rs::ProcessorConfig`, then
  `processor/estimate.rs::Estimate::record`, then
  `processor/estimate/seam.rs::SeamDetector`.
- Full settings receive the derived flag before materialization/checkpointing;
  configuration-view refreshes and export clones retain it. Delete the obsolete
  `options/config_export/transform.rs::apply_scarf_joint_seam` and its call:
  passing empty objects was a no-op, not a useful delegation. Do not replace it
  with another formula or empty wrapper. Config export simply clones the effective
  settings; no writer/helper/test interface changes are required.
- The current exporter-only default check and the ungated detector are temporary
  compatibility shells around the cited PrintApply/SeamsDetector concepts.
  Replace the duplicate default check, not keep it alongside the new owner.
- Do NOT run `transformed_for_export` on processor settings: that routine also
  changes extruder/flush/oracle state, none of which belongs to this correction.
- In `associate`, port only the guarded previous-Z refresh into the existing
  reached external-wall branch. Retain float vertex storage, the existing
  0.0625f test, activation/closure sequencing and last-vertex updates.
- Cache admission, cache IDs, scheduled times, accumulation, elapsed snapshots,
  progress percentages/remaining minutes and output formatting remain untouched.

No new public option, candidate crate, platform I/O, geometry guess, duration
multiplier, unsupported guard, ID blacklist, output patch or legacy fallback.

## First producer and consumer evidence

- Identical first367 movement/homing commands, including feed-only/no-op IDs;
  353 shared completed-command boundaries match by command and exact endpoint.
- All completed first_vertex states match through ID324. Its remembered first is
  `[119.7750015258789,100.30500030517578,0.20000000298023224]`.
- ID325 is `G1 X119.775 Y102.669 Z.224 E.00481` (Rust raw input line391).
  Immediately before it, the previous stored vertex is
  `[119.7750015258789,101.4469985961914,0.210999995470047]` on both sides.
  Native layer-tag mode is true and refreshes first to that previous vertex;
  Rust retains the lower first. This is the first persistent post-command
  first_vertex divergence, exactly the missing upstream transition.
- At closure ID361 Native inserts a Seam vertex; Rust does not. All99 extra
  Rust cache entries are Native Seam IDs. The18184 common cache IDs/f32 elapsed
  values and total double `1597.7582713937154` are exactly equal.
- At the visible residual, Rust lookup5327 hits an extra travel entry after
  feed-only ID5328 and emits `M73 P29 R18`. Native omits those cache IDs and emits
  the same marker after SparseInfill ID5330 using common lookup5329.
- Archive raw flag is false with global scarf external. Native PrintApply derives
  true; entry-DWARF shows the current Rust processor still receives false.
  The existing export transform mutates only a clone. Merely reading the raw
  flag and adding a guarded branch would not correct this witness.

All valid GDB runs use `may-call-functions off`, field/memory reads, normal
execution and normal exits, with complete output equal to non-GDB output.
Rust source214 is partially inlined and is NOT true call-entry evidence; use
Rust230 completed states and adjacent completed ID324 as the real before325
boundary. Native requested4203 resolves4204 after the setter, never before-state.
Rejected unavailable/zero-length-read early probes are not acceptance evidence.

## Native-first public RED

Freeze four input-only archives: the two EXISTING Native-exported archives
below, unchanged, plus the independently admitted GUI part pair below:

| Role on d9 main | Input | SHA256 |
|---|---|---|
| RED target | min50 wipe-OFF | `6282599cada020faa3984a9442879b8c4b6c13f6e41b8ebeaea930546723dacc` |
| Sole-key PASS control | min50 wipe-ON | `0047faf8b3c4c2f500d101aa2acc4df9ebebb5a075971598efcf5f984e16799b` |

Their sole settings JSON delta is `wipe_before_external_loop`. Do not confuse
these roles with the older pre-external-wipe matrix: ON now completely passes;
OFF is the new lifecycle RED. No setting search or motion-only acceptance.

Reuse actual main `runner/native_api_cli.rs::NativeCase`; the existing
`native_stream_case` adapter already delegates to that single execution engine
and retains Native double replay. Do not add another engine, normalizer or fork.
For each input and official/seeded mode, require fresh stable Native2 BEFORE
public byte API and real CLI, then complete ordered generator-only Native/API
and Native/CLI comparisons, plus raw API/CLI equality and successful runtime
statuses. Keep every M73 and all metadata. Retain repeatable artifacts.

Discovery already confirms fresh Native2 per mode stable and CLI OFF FAIL / ON
complete PASS. The public byte API RED is NOT yet claimed. Its failure must be a
real complete-stream mismatch, not rejection, unsupported options or a CLI error.
The canonical helper panics on a Native mismatch before its final raw assertion;
independently audit both emitted raw files on RED rather than pretending the
later assertion ran.

## GUI-reachable part/volume witness admission

Additionally evaluate, READ-ONLY first, a GUI-reachable archive with global
`seam_slope_type=none` and a part/volume `seam_slope_type=external` override.
Use the existing Native project metadata/export boundary and preserve geometry,
settings and override provenance; verify Native consumes the part override.
This is not a search over unrelated settings to hide another residual.

Only admit it to the RED matrix after Native double stability and a real
Native/current-Ares full-stream divergence are observed. If it does NOT diverge,
record that fact here as deferred; do not fabricate a RED or add a guard.
Any admitted target needs its declared single-variable complete-PASS neighbor.
Admitted after read-only Native/current-Ares verification on the unchanged base:

| Role | GUI setting (global none) | SHA256 |
|---|---|---|
| RED target | part external | `19aedecce54f64e70b9cd02752df43a254e300064abb7923b825b5ea480034ca` |
| Sole-key PASS control | part none | `44e347dcf43536d940d203a8035eca8ff6e10313f6fe6ed2da67a080e728e797` |

Only `Metadata/model_settings.config` differs, at the single part
`seam_slope_type` attribute. Global settings, geometry and all other payloads
match. The GUI-equivalent metadata seeds were re-exported by Native; only the
Native-exported input-only bytes may be frozen, not the temporary seeds.
GUI provenance: `GUI_ObjectSettings.cpp:241-249`, `Tab.cpp:2337`,
`PrintConfig.hpp:1232`; Native loader/exporter `bbs_3mf.cpp:5116,5264,7878-7879`.

Fresh official/seeded Native2 is stable for both inputs. Part external has four
whole-stream line differences against current CLI (the three M73 displacement
lines and Native `has_scarf_joint_seam=1` versus Ares 0); part none completely
passes in both modes. This pair therefore enters the formal public RED matrix,
not deferred and not admitted on a motion-only claim. No public API
qualification is claimed from discovery.

The full RED matrix is four cases: two strict targets and two sole-key complete
PASS controls. Object/layer-range any_of behavior is STILL required in the
implementation: these witnesses do not justify a default/volume-only subset.

## Included / deferred and architecture constraints

Included: the complete source any_of flag derivation with one owner, its existing
configuration/export/processor handoffs, and the guarded previous-Z transition.
This preserves the source-defined Seam vertex/cache association and resulting
M73 lookup without changing either consumer.

Deferred: independent planner/time arithmetic, arc attribution, OptionsZ,
actual-speed insertions, percent/remaining formulas, scarf/wipe geometry,
seam-placement occlusion/embedding/overhang optimization, unrelated export
transforms and wider GUI behavior. No self-designed Ares slicing/timing pipeline.

All changed text files must be <400 LOC. Runner responsibilities were already
split; any later registration growth must respect that boundary without copying
logic. Core remains byte-oriented and portable to WASM/Windows/macOS/Linux.

Sequence: genuine public RED -> production-only GREEN. Any strict residual
means stop, not comparator filtering or a partial-GREEN claim.
