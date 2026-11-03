# J1 startup: emitted-tool usage mask, not initial-tool selection

Status: SPEC-REVIEW requested; no RED/GREEN implementation authorized yet.

## Source-owned rewrite boundary

Pinned OrcaSlicer revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This slice ports the by-layer `GCode::do_export` runtime placeholder
`is_extruder_used` population in `src/libslic3r/GCode.cpp:2901–2904`.
It is not a new Ares startup policy, a seam repair, or an initial-tool repair.

Upstream sources:

| Source | Ownership |
|---|---|
| `GCode.cpp:2730–2762` | Sequential/by-object local ToolOrdering construction and writer registration from `Print::extruders()` at2760 |
| `GCode.cpp:2765–2791` | By-layer reuse of `print.tool_ordering()`, initial filament selection, writer registration from `tool_ordering.all_extruders()` |
| `GCode.cpp:2845–2846` | Selected initial logical filament assigned to `initial_tool` and `initial_extruder` |
| `GCode.cpp:2901–2904` | Zero-filled bool vector, length `max(MAXIMUM_EXTRUDER_NUMBER, filament_diameter.size())`; mark each ID from the local `tool_ordering.all_extruders()` |
| `GCode/ToolOrdering.hpp:216–224` | First-printing and all-printing extruder accessors |
| `GCode/ToolOrdering.cpp:921–951` | First nonempty layer gives first printing tool; all layer tools are aggregated/deduplicated, then optionally reordered for priming |
| `Print.cpp:523–548` | `Print::extruders()` combines configured object/support candidates, custom tool switches and an applicable explicit tower filament |

### Non-negotiable distinction: writer IDs are not the mask source

The two writer-registration branches must remain distinct:

- by-layer: `set_extruders(tool_ordering.all_extruders())` at2791;
- by-object: `set_extruders(print.extruders())` at2760.

After their join, upstream mask population at2901–2904 always uses
`tool_ordering.all_extruders()`. In by-object mode this means the local
ordering present at that program point, NOT the writer's registered
`Print::extruders()` union and NOT a synthetic global union of all objects.
Do not replace this distinction with a single writer-ID-based rule.

This J1 implementation scope is **by-layer only**. The exact by-object
local-ordering/export lifecycle is deferred, with an unchanged-output
control required. Its writer registration and existing mask behavior must
not be silently changed while repairing this J1 by-layer case.

## Frozen finding and corrected diagnosis

Finding baseline: `325ba112d1856d0178a4672068d3f8784afdf72e`.
Current integrated baseline when this spec was written:
`0164565498d3cc49f087611b56ec66608705d64a`.

Retained evidence: `/tmp/worker6-seam-B-startup-finding.md` and
`/tmp/worker6-seam-B-startup-finding/`. All eight checksum-bound files were
read-only verified before planning this slice.

- Input SHA256: `7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4`.
- Native output: `a06ecd924edbb729836bb835db523f9c0dc0b87495aab74579cd108377f9253e`.
- Ares output: `009ef1fb2409d7e34064a949ae04f6fbff5af22995f0824c6603bc0a62ed0129`.
- Receipt binds this input and Native output, seeded Native59fb88a2…,
  resources286fab66…, seed0, CPU17. This historical capture is not renamed
  a fresh CPU0 run or substituted for the approved future RED.

Both frozen streams already print `; --- initial_extruder: 1` and start
with T1. Native renders `; --- T0: false`, Ares `; --- T0: true`; T1 is true
in both. Ares consequently emits extra T0 startup preheat/flush commands
and a flush move to X0 before returning to T1. Native only flushes the
used tool and its corresponding move reaches X324.

The true complete-file first difference remains M73 at byte461/line18,
Native `M73 P0 R2` versus Ares `M73 P0 R3`. Do not discard M73, change the
stock startup script, or substitute a command-only comparison to make the
finding pass. The bool/tool/flush witness diagnoses the symptom; it does
not supersede the whole-file comparator.

### Ares source chain

- `gcode_emit.rs`: builds `schedule.tool_ordering`, resolves initial metadata
  in `machine::first_filaments::resolve`, registers ordering IDs in the
  writer, and then renders machine startup.
- `tool_ordering.rs::initial_extruder` already chooses the first-printing
  logical filament (with its existing sequential/Type2 priming branches).
- `machine/first_filaments.rs::resolve/insert/insert_machine` already carries
  the initial logical filament into `initial_tool` and `initial_extruder`.
- `machine/start_config.rs:68–69` refreshes state and installs that metadata
  AFTER `placeholders::base_config`; the initial-tool connection is present.
- `placeholders.rs::insert_runtime_placeholders` instead builds
  `is_extruder_used` from `collect_project_object_extruders`.
- `project_slice/extruders.rs:23–38` adds the object's/volume's configured
  extruder even when explicit roles no longer print with that extruder.
  The frozen XML has object extruder1, while all enabled perimeter roles
  select2; its unused default therefore contaminates the mask with tool0.

The configured candidate collector has other consumers, including nozzle
validation, support selection and fan metadata. They are not this export
placeholder's owner and must not be rewritten or deleted in this slice.

## Rust destination and scope

Owning boundary: `crates/ares-core/src/project_slice/gcode_emit/` runtime
export metadata, specifically `placeholders.rs`,
`machine/first_filaments.rs` and, only if needed to install that metadata,
`motion/writer_state.rs` / `machine/start_config.rs`.

Included:

1. By-layer emitted usage comes from the existing finalized
   `ToolOrdering::all_extruders()`; no second geometry scan, config candidate
   union, model-extruder inference or active-tool-only substitute.
2. Preserve zero-based LOGICAL filament IDs. Do not remap the mask through
   `filament_map` or `physical_extruder_map`; the startup script indexes the
   logical filament vector.
3. Preserve vector width `max(64, filament_diameter.size())`, false unused
   slots and existing bool rendering. No shrink to number of used tools.
4. Install the snapshot in the by-layer machine-start template context and
   keep stateful export contexts that reseed/refresh the same runtime
   metadata consistent. End/layer contexts must not reinfer an unused
   default tool from the model.
5. Remove `collect_project_object_extruders` as the effective by-layer
   runtime mask source. Do not retain it as a fallback for by-layer output.
   Reuse the existing export metadata shell for the upstream snapshot;
   no independent Ares tool-selection service or new public test API.

The shared base-placeholder builder currently also serves pre-ordering
`file_start_gcode` and the deferred sequential path. Treat their existing
scaffolding as a temporary compatibility shell, not proof of upstream
runtime parity. Do not expand J1 to redesign those phases or delete a
collector still used outside the by-layer runtime-mask path. Keep those
consumers unchanged unless a reviewed source boundary explicitly requires
otherwise; explain the precise installation point in the GREEN diff.

Deferred:

- by-object local-ordering traversal/mask parity and the distinct sequential
  writer registration lifecycle;
- changes to tool ordering, initial-tool/initial-no-support selection,
  physical remapping, priming, support/tower candidate discovery;
- validation/nozzle used-set semantics and fan candidate calculations;
- custom start-script design, template expression semantics, temperature,
  retraction, cooling, geometry, seam B, time estimation and M73 formatting;
- pre-ordering file-start-template placeholder availability.

Do not hide an unrelated divergence discovered after the mask symptom is
fixed. Retain complete bytes and report a separate residual; broader GREEN
requires its own source-cited spec/RED approval.

## TDD and evidence gates

No test/production implementation before SPEC-REVIEW approval. No new
build or Native task until BROWSER-END.

Confirmed seam: public project-byte slicing API / existing CLI parity
harness, with real pinned OrcaSlicer references and complete output bytes.
No private emitter access, panic instrumentation or Native rebuild.

After approval:

1. Freeze the original J1 input unchanged as the primary regression. Add
   a test-only E2E using the stock startup script and verify exact input
   SHA and the fresh Native receipt. Assert complete bytes with only
   generated-by identity normalized; retain input, streams, manifest,
   command/deadline record and the mask/tool/flush witness.
2. Establish fresh Native complete-byte RED on current clean production
   BEFORE a fix: CPU0/seed0, runs1, fresh per-case cache, shared300s deadline,
   owned-descendant cleanup. Prebuild outside the deadline with
   CARGO_INCREMENTAL=0 and16-thread limits. Existing finding is supporting
   evidence, not the independent RED gate. Submit test-only RED hash for
   approval.
3. Freeze controls before their first Native comparisons: ordinary tool1
   only; actually-used tools1+2; and a sequential/by-object unchanged-output
   case. Prefer existing independently byte-green controls where they
   expose the intended branch; do not tune geometry/settings after Native
   comparison. If a required control has an unrelated residual, report it
   rather than relaxing assertions or promoting it to PASS.

   **Coordinator ruling after the independent first RED run:** the three
   frozen controls have unrelated existing whole-file differences. Role1
   and by-object first differ at byte2449/line165 (Native M73 P41 versus
   Ares WIPE_START); both-used first differs at byte2384/line189 (Native
   FORCE_RESUME_FAN_SPEED versus Ares blank/change T0 -> T1). Retain those
   exact inputs and outputs as separate findings. For these three controls
   only, assert exact usage mask, initial logical tool and literal tool/flush
   command sequence from the independently captured Native output; do not
   perform a complete-byte comparison or label their whole streams PASS.
   The primary role2 case MUST still assert complete bytes; if the mask fix
   exposes a later residual, report it unchanged and stop rather than strip
   any output. Frozen findings:
   `${TMP_ROOT}/ares-j1-mask/findings/wipe-progress/FINDING.md` and
   `${TMP_ROOT}/ares-j1-mask/findings/filament-start-transition/FINDING.md`.
   By-object unchanged coverage here is witness-only, not full parity.
4. Only after exact-hash RED approval implement the minimal runtime mask
   connection. Keep initial-tool expectations and stock template unchanged.
5. Fresh base/candidate full seeded, directed RED+controls, strict Clippy
   `-D warnings`, rustfmt, workspace nextest and WASM check. Because mask
   metadata is shared by stateful templates, use fresh full official unless
   the coordinator explicitly approves a source-justified subset.
6. Compare named raw failure sets: candidate failures must be a subset of
   base, not merely an equal count. Audit same-input/payload output
   invariance, per-case receipt binding and RED/GREEN input invariance.
7. Submit exact candidate hash and REVIEW-REQUEST. Fetch/rebase latest main,
   satisfy current rebase gates, then nonforce fast-forward push only with
   approval and independently confirm the remote hash.

## Exit criteria

The primary frozen stock-J1 whole-byte RED becomes PASS with tool0 false,
tool1 true, initial_extruder1 unchanged and no unused-tool flush. Controls
and gates satisfy the above evidence policy; no comparison relaxation,
fixture tuning, ignored failing test, retry-until-success or unrelated
pipeline repair. Sequential/by-object behavior remains unchanged and is
explicitly not claimed upstream-complete by this by-layer slice.
