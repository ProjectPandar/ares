# GCodeProcessor sloped-loop seam lifecycle and effective scarf flag

Status: COORD-approved rewrite scope; docs-only SPEC first. Public RED is next;
production GREEN must wait for a separate approval of the RED RR. Base is
`d9d01e2e86ecdb3005e1c4be604b6fd913fa9f9b`. No FULL window is open.

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

The effective flag is an approved necessary dependency of this SAME slice,
not a separately deferred prerequisite: without it this witness cannot turn green.

## Exact Rust destination and single owner

- Add the shared source-owned derivation under
  `project/effective_config/scarf_joint_seam.rs`, exposed within ares-core.
  Its inputs are the effective full settings and the original model objects'
  typed object/volume/layer-range option sources. `ProjectObject::region_overrides`,
  `volumes`, and `layer_config_ranges` preserve the three source paths. Reuse
  the existing typed override/inheritance access, without resolving or cloning
  complete per-volume configs merely to inspect this one enum.
- The shared function is the ONLY owner of the PrintApply any_of expression,
  including explicit incoming true. Prefer borrowed sources and lazy any_of.
  Do not reconstruct it independently in a formatter, processor or GUI adapter.
- Feed the derived flag through existing effective-config/slicing handoffs to
  `gcode_emit/processor.rs::ProcessorConfig`, then
  `processor/estimate.rs::Estimate::record`, then
  `processor/estimate/seam.rs::SeamDetector`.
- `options/config_export/transform.rs::apply_scarf_joint_seam` MUST delegate to
  that SAME shared derivation. Thread the required model-source context through
  existing config export calls in `project_slice/state.rs` and
  `gcode_emit/tool_ordering/recommend.rs` as needed. Configuration-view refreshes
  must not lose the derived flag. Existing config-only export call sites must
  supply an explicit source context rather than invent a default-only fallback.
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

Read-only evidence root:
`/home/indexyz/ares-evidence/w6-m73-d9-readonly/`.
`ordered-state-proof.json` and `spec-draft.md` retain the detailed measurements.

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

Freeze the two EXISTING Native-exported input-only archives, unchanged:

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
Current status at docs-only submission: candidate not yet evaluated or frozen.
Object/layer-range any_of behavior is STILL required in the implementation even
if its additional output witness is deferred: no default-only substitute.

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

Sequence: this docs-only commit -> genuine public RED commit/RR -> STOP for COORD
approval -> production-only GREEN if separately authorized. Any strict residual
means STOP, not comparator filtering or a partial-GREEN claim. FULL waits for a
new window. The isolated obsolete test deletion remains unchanged with the
user's decision still pending. No push or self-merge.
