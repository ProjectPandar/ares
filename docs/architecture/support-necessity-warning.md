# Support-necessity warning — PrintObject source rewrite boundary

Status: ARD and public CLI RED approved. The core-only port was explicitly
authorized while waiting for its success-report writer prerequisite; the real
public RED now establishes missing warning fields in two successful reports.
Producer and single-writer adapter GREEN work is authorized; qualification and
review remain required before any behavior-parity claim or slice completion.
Source: OrcaSlicer345efec64c7b95a05ce5cf96df0dde66a158835c.
Initial analysis base: mainf4ea41332cdcb7eadf9adbc7f7c1936c0ad7e9c5.
The RED commit must be based on the separately approved beta CLI warnings
writer's posted rebased tip; do not recreate that transport in this slice.

## Owning upstream behavior, not a new warning heuristic

`PrintObject::generate_support_material` (`PrintObject.cpp:961-1008`) checks
support necessity during the support-material step, before support generation:

1. Enter only when `set_started(posSupportMaterial)` succeeds, and clear old
   support layers. Do not emit a duplicate warning per extrusion/layer/copy.
2. Check only if **!has_support() && !print.get_no_check_flag()** (`:966`).
   `Print.hpp:506` defines has_support as `enable_support || enforce_support_layers>0`.
   Thus a positive enforced-layer count suppresses this check even with support
   disabled. **Raft alone does not suppress it**. The separate generation guard
   (`:999`) also covers raft and must not replace the warning guard.
3. `PrintObject::is_support_necessary` (`:5289-5303`) constructs TreeSupport,
   **forces its effective support type to TreeAuto**, calls
   `detect_overhangs(true)`, then clears temporary support layers again.
4. Return SharpTail first if has_sharp_tails; otherwise Cantilever only if
   has_cantilever **and max_cantilever_dist>scale_(6)**. Strict inequality,
   scaled units. Equality6mm and the detector's intermediate3mm threshold
   are not warning thresholds. Otherwise NoNeedSupp.
5. Emit `WarningLevel::NON_CRITICAL`, notification `SlicingNeedSupportOn`,
   with **model_object()->name**, not a generated instance label (`:975-986`).

`SupportNecessaryType` (`Print.hpp:87-92`) also names LargeOverhang, and the
caller's message map has text for it. **This pinned is_support_necessary never
returns LargeOverhang**. Do not invent a third detector/area threshold merely
because a message exists. Included reachable text is exactly:

- `It seems object %1% has floating regions. Please re-orient the object or enable support generation.`
- `It seems object %1% has a floating cantilever. Please re-orient the object or enable support generation.`

The adjacent `large overhangs` message remains source vocabulary, not an exercised
or newly reachable classifier in this slice.

## TreeSupport check-only phase and retained Rust substrate

Upstream `Support/TreeSupport.cpp::detect_overhangs`:

- `:739-747` permits check_support_necessity even when configured tree support
  is disabled; it clears support/preview caches. Effective forced TreeAuto
  passes `is_tree(stype)` (`:876`) and automatic feature detection gates.
- `:756-773,878-895` resolves object line_width against nozzle0, angle
  `(threshold>EPSILON ? threshold+1 :30)` capped89°, prepares extrudable
  silhouettes and resets maximum distance. The sharp-tail global is true
  (`libslic3r.h:116`); do not bind it to a UI-only option.
- `:899-1079` performs the first pass. The first layer may seed sharp-tail
  regions but does not itself set has_sharp_tails. Floating-region detection
  and cantilever flags are set by later-layer tests. The30second/>100region
  guard and source mutation of detection flags are part of this kernel.
- Cantilevers update max distance only beyondscale_(3) (`:1062-1076`).
  `Polygon::distance_to` inherits `MultiPoint.hpp:78-83`: minimum distance
  to polygon **vertices**, not line segments. Preserve that source quirk.
- **`:1083-1084` returns immediately for check_support_necessity**. Do not run
  sharp-tail propagation, cluster cleanup, blockers/enforcers/bridge filtering,
  final type assignment or support node/toolpath generation for the warning.

Existing Ares scaffold already ports much of that first-pass geometry:

| Existing Rust boundary | Retained meaning / missing coupling |
|---|---|
| `project_slice/support_material/tree/overhangs.rs::SmartOverhangs` | fields has_sharp_tails, has_cantilever and max_cantilever_distance; false/false/0 defaults and layer silhouette preparation |
| `tree/overhangs/first_pass.rs::detect_layer` | automatic-type guard, elapsed/>100 guard, floating_sharp_tails and detect_cantilevers; first-pass flags and nearest-vertex distance already exist |
| `tree/overhangs/finalize.rs::finalize_layer` | adds more sharp-tail/cantilever state, but this is **after the upstream check-only early return** and cannot be used to classify this warning |
| `tree/non_organic/generation.rs:51-78`, `tree/organic/generation.rs:173-186` | currently use SmartOverhangs for enabled support generation, then propagate/finalize; not the disabled-support check-only invocation |
| `support_material/project.rs:82-85` | currently returns early for disabled support/enforced-layer0, exactly where a separate source-owned necessity check is missing |

Do not rewrite these as an Ares-owned overhang detector or duplicate the geometry
algorithm in a warning utility. The existing enabled-generator path remains a
compatibility shell around TreeSupport; add the source's **check-only consumer**
and effective-type selection, rather than reading unrelated finalized outputs.
The forced type belongs to this invocation; do not mutate user's support settings
or clone/reconfigure a project to force an unsupported public result.

## Rust destination and non-negotiable architecture

- **Classifier/guard owner:** a small `project_slice/support_material/necessity.rs`
  owning PrintObject::is_support_necessary and its caller guard. Reuse current
  sliced-object inputs and SmartOverhangs first-pass methods. The insertion point
  is the existing support-material stage, before its disabled-support return,
  not validation, G-code parsing, viewer rendering or an STL bounding-box guess.
- **Source check-only invocation:** `tree/overhangs.rs` and `first_pass.rs` need
  only the minimum effective TreeAuto/check-mode coupling. No full generation,
  support install or post-first-pass finalize for this path. Scratch layers
  must not leak into actual installed support or a later slice.
- **Warning data:** model the source PrintObject warning status, not SliceError
  or GCodeProcessorResult. Source `PrintBase.hpp:305-333,674-679` deduplicates
  notification IDs within the active object step and sends an update only for
  a new/changed/current warning; fresh slice calls must not inherit stale flags.
  Do not add a global warning store or recreate incremental PrintState machinery
  when the existing fresh-slice boundary already supplies the lifecycle.
- **Transport:** use the posted beta CLI warnings writer as the sole adapter
  for class `slicing_warning_non_critical`, `plate_id`, `text`
  (`OrcaSlicer.cpp:7150-7160`). Native records it before G-code export; do not
  drop it on a later export error. `PrintBase::SlicingStatus` is the core source
  concept; CLI JSON/stderr formatting remains in ares-cli, never core I/O.
  Bind the smallest required public status/result carrier after inspecting the
  landed writer seam; do not invent a second warnings/reporting framework.
  Proposed core destination is `print/warnings.rs` for the source
  PrintStateBase warning record, with the existing `ProcessedGCode` slice-output
  shell carrying those records to the adapter. Its `GCodeProcessorResult` remains
  processor data and must not own support warnings. The byte-only convenience
  slice wrappers may still return bytes; the warning-aware CLI must use the
  processed slice result instead of discarding statuses. Exact CLI plumbing is
  bound to the landed writer's success path before the implementation RR.
- At the analysis base no public slicing-warning carrier or no_check API exists.
  Do **not** equate validate_print=false with Native no_check=true: the flags
  have different source owners. Do not add a new CLI no-check/strict option in
  this classifier slice. Ordinary current requests correspond to the source's
  default no_check=false (`PrintBase.hpp:568`). Document the missing optional
  CLI switch separately rather than silently suppress the warning.
- `NON_CRITICAL` does not become a fatal core error or alter G-code. Critical
  warning classes, new strict-mode behavior and unrelated output/report fields
  remain their owning adapter slices. Tier1 browser WASM/Windows/macOS/Linux
  all use the same pure core geometry/status data; no filesystem/terminal in core.
- No legacy fallback, feature flag, invented supported-area threshold, mesh
  label match, canned warning, copied Native kernel or approximate detector.
  Existing ordering, scale/precision and per-object settings are authoritative.

## Public RED plan and failure inventory — before test code

The RED commit must wait for both the warnings writer's **posted rebased tip**
and its success-path follow-up. Its old commit ID alone is not authority to pick
an obsolete base or duplicate a writer. The ARD can be reviewed independently.

### Chosen public seam: stack on the success writer

The retained real witness is the committed
`tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf`, SHA256
`2dbf797da35adf51f085c397cfa959364fd7411a73ef52ec687d04dd480e7342`.
The actual pinned Native `--slice 0 --outputdir ... <fixture>` exits0 with empty
stdout/stderr. Its result.json has exactly one `slicing_warning_non_critical`
record, plate1, for **overhang.obj**, and the same full text in
`sliced_plates[0].warning_message`: floating cantilever. This is not a stderr
warning witness and not a rejected-slice fixture.

At the analysis base `ProcessedGCode` exposes G-code, processor result and print
statistics, but **no slicing-warning output**. Creating a new core seam before
the feature, or asserting that a success report file is missing, would not
isolate this detection gap. Therefore choose the existing public **CLI result
JSON** boundary, after the success writer lands underneath the RED commit:

1. Generate the same real fixture Native result at runtime using the pinned
   official beta CLI, retaining command, exit, complete stdout/stderr/result and
   full source/input/ELF identities. No saved diagnostic JSON as expected values.
2. Run the Ares public CLI on the identical fixture/plate. Require both runs to
   succeed and both result.json files to exist and parse with the landed writer
   schema. A missing writer/report or wrong exit is a prerequisite failure,
   **not the classifier RED**.
3. Compare the complete `warnings[]` array and every selected plate's
   `(id, warning_message)` projection exactly against that actual Native JSON.
   Require the Native witness to really contain the noncritical class/plate/text,
   so an unexpectedly empty Native run cannot produce a false GREEN. No invented
   message fragment, ignored warning entries or relaxed string comparison.
4. Keep existing whole-result/G-code/stderr comparisons unchanged. The warning
   projection is the explicitly scoped new public warning test, not a waiver for
   other report metadata. For this real fixture stderr is empty on Native;
   do not assert a fabricated diagnostic warning line there.

The known Native example text is source vocabulary/provenance, not an embedded
test expectation substituted for runtime Native. Its source object name and
plate ID must travel through the actual warning-producing slice.

Confirmed external observation boundary is a real public CLI slice and Native
result.json, not private SmartOverhangs fields or a scratch detector. Bind input
bytes, source and ELF identity, command/exit, exact object name, plate and text.
Expected message/class comes from the actual Native execution. No production
changes, new diagnostic hook or private-state test is implied by this plan.

A success Native result.json cannot serve as a trivial whole-file RED if the
landed Ares writer still defers success reports. Likewise a missing report or
compiler error is **not this classifier RED**. Use the agreed landed writer
seam or public warning-bearing stderr/status channel so the failure is genuinely
missing/different SlicingNeedSupportOn data. Confirm that seam from the writer
handoff before writing the test. Keep existing whole-output comparisons intact;
any broader unported reporting channel remains visible, not waived.

Ways this slice can fail:

- Check omitted because support generation exits early, or wrongly gated by
  validation, raft, configured NormalAuto/Manual type or support enable alone.
- Enforce-support layers>0 ignored, no_check conflated with validate_print,
  equality6mm promoted, intermediate3mm treated as warning threshold.
- SharpTail priority lost; layer0 seed counted as a warning or post-finalize
  flags used instead of first-pass flags; phantom LargeOverhang branch invented.
- Wrong line width/nozzle/threshold/scale/nearest-vertex semantics; elapsed and
  region-count guard changed; belt/tilt inputs treated as ordinary lower slices.
- Wrong object name, severity, notification/class/plate or duplicate warning
  per copy/layer; stale flags across independent slice calls.
- Warning lost on the CLI path or after a later export failure, or recorded only
  in a private debug hook. Detection must not generate actual support layers.

Start with **one real fixture / one runtime Native public RED**, through an
already-registered test leaf or dedicated public CLI integration test (no ROOT
registry edit). Only add a source-required control when the first witness shows
which guard branch is exercised. No horizontal bulk test suite or unit-after-code.

## Approved adapter connection

The success writer is the prerequisite boundary, not a second warning framework.
The existing report-aware `SlicedPlate` holds the source warning snapshot before
support generation/export, so the caller retains it on later slicing/export
errors as well as success. `ProcessedGCode.warnings` exposes the same source
records on the public core result. Neither is GCodeProcessor warning state.
The adapter uses `CliWarning::SlicingWarningNonCritical { plate_id, text }` in
its exhaustive single-writer match, appends it after load substitutions and sets
`SlicedPlateEntry.warning_message` to the last source warning text
(`OrcaSlicer.cpp:7143-7160`). JSON string escaping uses the existing serializer.
No stderr output is introduced: the actual successful cantilever witness has
empty stdout and stderr. The existing family-error report uses that same
warning snapshot and writer; no generic SliceError wrapper or global callback.

## Included, deferred and exit gate

Included proposal: exact caller guard, forced TreeAuto check-only first pass,
SharpTail/Cantilever decision and reachable warning record at the landed public
adapter seam. Before implementation, audit any missing kernel prerequisite on
the actual fixture instead of silently replacing it with a warning heuristic.

Deferred adjacent behavior: full tree-support generation changes, post-first-pass
finalize/bridge/blocker behavior, new strict/no-check CLI options, unrelated
critical classes, success-report writer work, belt/tilt kernel parity not already
present in the reused detector, background incremental scheduler behavior and
unreachable LargeOverhang. These deferrals are not permission for a wrong result
or a fallback on an exercised input: if the actual fixture needs missing belt/
tilt/other prerequisite, stop and submit that source boundary for review first.

Exit for this phase: source/retained-state audit, ARD, one real public RED with
verifiable runtime warning evidence, then SPEC/RED RR. **No production port before
review authorization.** Keep nextest/fmt/workspace-all-target-all-feature clippy
`-D warnings`, strict comparisons, original Native-required policy and artifacts
outside Git. No committed Native binary, G-code, unpacked resources or goldens.
