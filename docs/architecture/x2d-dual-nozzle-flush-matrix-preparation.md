# X2D-FM: source-owned dual-nozzle flush-matrix preparation

## Status and decision

Approved boundary, revised after source tracing and coordinator's system-boundary
ruling (2026-10-07). Production is uncommitted and not accepted; initial RED
`65336cb5` remains historical evidence but its undersized scalar type vector
is not a defined-input calculator acceptance witness. **Its original Ares-error /
Native-success divergence remains unresolved**, separately tracked in
[O8-X2D-SCALAR](o8-x2d-scalar-flush-context.md); this slice must not claim to fix
that reported input. Fully specified missing-matrix reconstruction is the FM
RED. Generic materialization RED moves to a separate slice.
Predecessor VG1/UsedFilaments is integrated at `9e3e8183777d465f9134361eb26c0a27d82e82c7`.
This is #55 / `X2D-FM` in `${TMP_ROOT}/ares-matrix-clusters.md`, independently owned by
pi-lint. It is configuration preparation, not a processor timing patch or an
Ares-designed purge pipeline.

Upstream revision: OrcaSlicer v2.4.2
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Paths and line numbers below are
relative to `OrcaSlicer/src/` at that revision.

**Non-negotiable:** port the preparation that precedes
`libslic3r/GCode.cpp:5596–5614::GCode::append_full_config`, not a relaxed matrix
size guard. The final matrix is nozzle-major: each nozzle owns one row-major
`filament_count × filament_count` segment. Export scales each segment by that
nozzle's multiplier with `std::round`; it does not create missing segments.
Preserve the upstream one-filament exception and exact multi-filament error
`Flush volumes matrix do not match to the correct size!`.

## Evidence and qualification

Historical case `${TMP_ROOT}/ares-vg1-red-artifacts-matrix/case-Uut51r/` binds the
repository cube10 and two logical palette/map entries to the exported X2D
profile, without VG1. Input `input.3mf` is15830bytes, SHA256
`04c2a738e93d5d161943d7f29dba6141ee1f6c6cd90512cf6ecc8c97eeec2c8d`.
Raw settings include two0.4mm nozzles, colours `#F2754E,#0000FF`, map `1,1`,
`Auto For Flush`, empty AMS, `nozzle_volume_type=[Standard]`, multiplier `[0.3]`,
and matrix `[0,140,140,0]`. Ares rejects; the retained Native stream contains
multiplier `0.3,1`, AMS `1#0|4#1;1#0|4#1` and matrix
`0,102,186,0,0,339,621,0` (164039bytes, SHA256
`bc154c764965c8015a37b46a4bc58b864353e1d11c45455f943e7f8094badcec`).

This is diagnostic evidence, **not current-base acceptance**: its manifest
explicitly labels the cached Native producer identity unknown. Before production
edits, regenerate a fresh identified seeded reference and an official reference
from the same bytes on current main, retaining commands, resources, hashes,
exit status and effective config. Reproduce the Ares error on that base. Do not
promote old cached bytes to a trusted oracle or invent an ordered-byte difference
when Ares produced no stream. External X2D models remain forensic only; no
external model or generated forensic G-code is committed as a fixture.

## Exact owning source boundaries

| Source | Included contract |
| --- | --- |
| `OrcaSlicer.cpp:1708–1718,3022–3047` | Distinguish current project nozzle-volume-type vector from new CLI nozzle-volume types. Without a CLI override, new types default to one Standard per nozzle. Compare vectors, not only their first value: `[Standard]` differs from `[Standard,Standard]`. Retain pre-preparation state until the rebuild decision. |
| `OrcaSlicer.cpp:3340–3351` | Rebuild only when project colours exist and selected CLI colours, absent matrix, changed extruder count, or changed nozzle-volume-type vector requires it. Missing AMS, Manual mode, model name or a matrix size mismatch is **not** this predicate. |
| `OrcaSlicer.cpp:3353–3412` | Apply nonempty selected colour overrides, then use the final project palette and `filament_is_support`. Port the pure colour/config operations needed by this entry, not thumbnail/UI mutation. |
| `OrcaSlicer.cpp:3412–3465` | Use `get_min_flush_volumes(m_print_config,0)` as written; check support/palette cardinality, resize matrix to palette²×new nozzle count, resize multiplier preserving supplied prefix and fill new slots with1. Resolve each nozzle's `nozzle_flush_dataset` through printer-extruder-variant indexing. No blind matrix duplication or zero padding as the result. |
| `OrcaSlicer.cpp:3467–3503` | Recompute every nozzle's matrix. Diagonal0; to-support230; other pairs use `FlushVolCalculator(min[from],max,dataset[nozzle])`; from-support floor700. Write segments back with `set_flush_volumes_matrix`. |
| `libslic3r/PrintConfig.hpp:2130–2153` | `get_flush_volumes_matrix` / `set_flush_volumes_matrix`: contiguous per-nozzle segment layout. These helpers split/store, not calculate colour-dependent volumes. |
| `libslic3r/FlushVolCalc.hpp`, `FlushVolCalc.cpp:10–140` | `FlushVolCalculator` and constants: alpha-zero treated as white, predictor precedence for dataset0 versus nonzero, HSV/luminance branch, source minimum, integer truncation, max20000, nonzero-dataset dark/light correction. Preserve source arithmetic types/order rather than deriving expected values from the forensic vector. |
| `libslic3r/FlushVolPredictor.hpp/.cpp` | `RGB2LAB`, `calc_color_distance`, `is_similar_color`, `FlushVolPredictor::predict`, `GenericFlushPredictor`: ordered first matching dataset colour within DeltaE threshold, directed pair lookup, supplied dataset selection and source-defined prediction miss branch. Required dependency, not an optional replacement with HSV-only calculation. |
| `slic3r/GUI/Plater.cpp:764–835` | Extract only pure `get_min_flush_volumes`: nozzle volume, machine/filament long-retraction selection,1.75mm area subtraction and integer conversion. No GUI/wx dependency. |
| `slic3r/GUI/BitmapCache.cpp:543–559`; `slic3r/Utils/ColorSpaceConvert.cpp:113ff::RGB2HSV` | Pure RGBA parsing and HSV arithmetic used by the cited CLI/calculator. Do not import rendering/cache code. |
| `OrcaSlicer.cpp:5993–6022` | Existing auto-map AMS default `1#0|4#1` per nozzle remains a separate later operation under mode<Manual, including replacement of an existing AMS mapping. Do not use this gate to decide matrix reconstruction. |
| `libslic3r/GCode.cpp:5596–5614` | Scale/round a valid full matrix exactly once for CONFIG_BLOCK; preserve errors/one-filament semantics. Existing `scale_flush_matrix` remains this operation, not preparation. |

The historical vector-change predicate explains Native entering reconstruction,
but does not establish the calculator's source input preconditions: its scalar
nozzle-volume type cannot cover two nozzles. Keep this observation diagnostic;
do not emulate its out-of-bounds reads or hardcode339/621. Acceptance uses two
explicit Standard entries and a genuinely absent matrix instead.

## Rust destination and replacement seam

All pure logic stays in `ares-core`; CLI/WASM only provide existing byte/options
entry points. No filesystem/resource lookup, terminal, wx, OpenGL, global mutable
predictor cache or new public slicing API in core.

- `crates/ares-core/src/options/flush_matrix.rs`: narrow preparation operation
  over typed `ProjectSettings` and raw-versus-resolved preparation context;
  owns the upstream predicate, cardinality, resize and nozzle-major computation.
- `options/flush_volume/`: source-cited calculator, predictor/colour math and
  immutable upstream `resources/flush/flush_data_{standard,dual_standard,dual_highflow}.txt`
  data needed for the supplied dataset ids. Keep Rust modules≤400LOC. Pin the
  source revision and resource hashes; compile/bundle data for all Tier1 targets,
  never silently omit it on WASM. Preserve data colour order and lookup semantics.
- `project/effective_config/phases.rs::resolve_with_observer`: minimal call at
  source preparation, **before the first normalize_fdm_1 / Print::apply-like
  passes**, using the project/raw and resolved input context already owned by
  core. Preserve source project immutability. The prepared unscaled matrix must
  feed full/runtime views and downstream existing tool ordering, not merely an
  export-only clone. Do not rerun preparation independently in every normalize
  pass or reclassify this as a new Ares lifecycle phase.
- `options/config_export/transform.rs`: replace the temporary one-palette matrix
  shell inside `prepare_multi_extruder_cli_defaults`; retain the separately cited
  AMS override and scarf/applied-map export operations. Keep `scale_flush_matrix`
  as the export-only round/scale seam. Remove superseded preparation rather than
  keeping fallback or two authorities.

The existing final map-recommendation and full/runtime views are compatibility
shells around upstream `Print::apply`/`ToolOrdering`, not license to patch their
algorithms. Their matrix consumers already exist in
`gcode_emit/tool_ordering/{collect,reorder}.rs` and `wipe_tower/planning.rs`;
this slice changes their prepared input only, not their ordering/emission code.
If the exact upstream preparation context cannot be preserved by this narrow
call (e.g. original/new preset identity lost), stop and revise the boundary with
coordinator; do not guess the predicate from matrix length or printer name.

## Failure inventory and test-first acceptance

Write E2E REDs/controls before production. Primary target uses repository cube10,
stock X2D profile and the minimum two-logical-filament binding from the existing
VG1 helper, with **no VG1**. The same fully bound3MF is supplied to Ares and both
identified Native modes. Palette/map application, producer success and complete
input/effective output artifacts are part of admission, not inferred from labels.

Discriminating cases:

1. Fully specified two-Standard X2D input with missing matrix: Native succeeds,
   base Ares exact matrix error; assert complete output after GREEN and the full
   effective matrix, multiplier and AMS values. The former scalar1-entry RED
   is retained as UB evidence, not an active correct-value acceptance test.
2. Two Standard input, unchanged nozzle count and supplied valid8-entry custom
   matrix: predicate false, retain custom values before one export scaling.
   Pair with malformed supplied matrix under the same false predicate: Native
   and Ares must still reject; do not repair every invalid shape.
3. Rebuild-trigger variants: absent matrix and changed nozzle-volume types;
   explicit colour/nozzle-count changes only through a faithfully represented
   existing entry. Inspect requested/applied/raw state to distinguish triggering
   from nontriggering variants. No synthetic internal branch tests as sole proof.
4. Manual versus auto modes and supplied versus empty AMS: matrix predicate
   independent of these, AMS later default independent of matrix reconstruction.
5. Directional colour pair reversal; equal-colour diagonals;7/9-character RGBA
   and alpha0; predictor hit versus miss; support source/destination rules;
   supplied dataset ids and per-nozzle selection; long-retraction active/inactive
   minima. These guard wrong arithmetic, wrong dataset and false row symmetry.
6. Multiplier prefix preservation, second-nozzle1 default, explicit two
   multipliers and rounding boundary; no double-scaling in runtime or export.
7. Existing single-nozzle/single-palette controls, H2D/X1C/X2D valid inputs and
   approved VG1/UsedFilaments/ColorChange controls remain unchanged unless an
   identified Native preparation difference requires a source-owned change.

Each end-to-end run retains project bytes/SHA, actual Native command and
producer/resources/seed identity, raw settings, effective config, complete
Native/Ares G-code or exact failure, manifest and repeatable comparison result.
Use existing full ordered-byte comparison with only its established generator
identity normalization. No CONFIG_BLOCK-only acceptance, matrix substitution,
footer heuristic, comparator relaxation, retry-until-success, ignored tests or
unknown-producer cache admission. Base/candidate common Ares streams are also
compared byte-for-byte without normalization. A new motion/ordering divergence
is reported with its first difference and source owner; this spec does not
preapprove repairing it in pi-tip's paths.

## Deferred boundary

No VG1/SYNC/timing/statistics change, physical nozzle/tool ordering rewrite,
flush-command emission, wipe-tower geometry, UI colour dialog, thumbnail
regeneration, external X2D model port, generic profile-cache architecture,
new crates or Native UB emulation. The calculator's **upstream** prediction miss
branch is included behavior, not a legacy Ares fallback. Required dependencies
cannot be dropped merely to make the initial size error disappear.

## Coordinator UB ruling and dataset-index dependency (2026-10-07)

Preserve `${TMP_ROOT}/ares-x2d-flush-matrix/green/seeded-focused/case-9whGha/`
unchanged as **upstream O8/UB adjacent**, not an Ares calculator numeric RED.
Its scalar `nozzle_volume_type=[Standard]` has only one element for two nozzles.
`OrcaSlicer.cpp:3460–3462` directly indexes `volume_types[eidx]` (and extruders),
with no vector broadcasting. Reading entry1 is out of bounds; the observed
Native second-head dataset0 behavior is undefined. Native's652/172 and the
observed Rust900/120 are retained evidence, **not correct-value contracts**.
Do not add a scalar-input expected-value assertion or emulate Native UB.

Coordinator explicitly approved changing dataset-dependent witnesses because
of this **UB input**, not to obtain a pass: specify both nozzle-volume types as
`[Standard,Standard]`, omit the matrix, and invoke the legitimate missing-matrix
rebuild branch. Palette/dataset inputs are taken from the pinned upstream data,
not selected by runtime outcome. Keep complete-stream comparisons. Under the
later any-vector coverage ruling, the initial scalar reconstruction RED is
retired from acceptance; the unchanged complete custom-matrix control remains.
All calculator/minimum/multiplier witnesses use fully specified nozzle vectors.

`PrintConfig.cpp:9004–9060::DynamicPrintConfig::get_index_for_extruder` remains
an FM dataset-selection dependency. However, the incomplete/generated-id E2E
also fails on pre-FM base9e3 with the same input and error, establishing an
existing generic materialization issue rather than an FM regression. Its RED
must move to a separately source-cited spec; FM does not depend on repairing
that generic boundary. No generic mapping API or emitter change is included.

## Defined boundary and predicate correction (coordinator ruling)

At the external configuration boundary, inspect the cells to be calculated.
Only a cell using the colour calculator needs a nozzle's type/dataset; if any
such dependency has an undersized `nozzle_volume_type`,
`printer_extruder_variant` or printer-id vector, preserve the supplied matrix
without reconstruction. Diagonals are always computable (`:3467–3503`); support
destinations also use their source constant rather than a dataset. Do not encode
this as a palette-count special case or choose a replacement dataset.

This supersedes the unconditional any-vector rule: source-independent cells must
still be prepared. In a one-colour matrix all cells are diagonal, so their exact
zeros and the multiplier-prefix resize (`:3412–3465`) require no type/dataset
lookup. Seven existing full-output REDs prove this: four seeded template-RNG
cases (`distributions_and_shared_stream`, `raise3d_{dual,left,right}`) and three
BBL SYNC cases (`positive_delay`, `without_sync`, `zero_delay`) retain Native/base
`flush_volumes_matrix=0,0` and `flush_multiplier=0.3,1`; the unconditional rule
incorrectly left a16-entry default matrix and multiplier0.3. Evidence is retained
under `${TMP_ROOT}/ares-x2d-flush-matrix/fresh-d789-identified/candidate-seeded.log`
and `rebase-1b175/prior-failure-inventory.json`. Use their complete-stream
comparisons, not new internal matrix assertions. This is source computation of
diagonals, not zero padding or a restored export-only shell.

The original two-colour X2D RED and the18 compact cases need off-diagonal
calculator cells, so their preservation rule is unchanged. Ordinary downstream
matrix validation and the exact existing error remain unchanged.

Source basis: `OrcaSlicer.cpp:3022–3047` creates the new Standard vector but
never resizes the current vectors. `:3460–3462` uses unchecked source indices.
`DynamicPrintConfig::update_values_to_printer_extruders` (`PrintConfig.cpp:9634ff`)
is called later, not before this calculation. Retained same-input diagnostic
`${TMP_ROOT}/ares-x2d-flush-matrix/trace/compact-aa.native.log` has flush computation
at lines68–72, an actual garbage `ExtruderType=25276`, identical before/after
matrix, and later update calls at lines126–128. The18 unique compact inputs
are inventoried in `trace/predicate-input-inventory.json`; existing whole-byte
controls, not new internal scalar-value assertions, cover their preservation.
`case-9whGha` remains untouched. Boundary preservation applies to unresolved calculator dependencies, not source
constants, and is not a legacy numeric fallback or an excuse to waive regressions.

The outer colour predicate at `:3340–3351` requires an actual project colour
option, not a later typed default. Sparse STL profile documents without
`filament_colour` must not reconstruct. Same-input seeded diagnostic
`trace/profile/00000.log` never enters flush calculation; the four existing
profile-entry whole-byte REDs show Native's original16-entry matrix versus the
erroneous reconstructed0. Check raw option presence before using typed colours;
do not add printer-name, AMS or matrix-length predicates.


## Short-lived old/new preparation context (coordinator-approved)

`OrcaSlicer.cpp:1704` captures the input nozzle count and current type option at
load time; `:3022–3047` compares this old side with the new CLI configuration
(and fills an absent/empty current type vector with Standard for the old count).
`:3340` independently tests existence of a CLI colour option. Keep exactly
three internal values: original input nozzle count, colour-override presence,
and current nozzle-volume-type vector. Capture before applying public/API
option overrides; selected profiles must retain the input side too, while STL
starts with the CLI's one-nozzle Standard current state.

Pass this value only through internal load/profile preparation to the single
matrix-preparation call. Do not put it in `ProjectSettings`, resolved configs,
public API or persistent project state. Drop it after preparation. This replaces
the insufficient predicate inferred only from an already-overlaid raw map; do
not compensate with shape/name heuristics. Source-independent multiplier resize
runs before declining unresolved calculator dependencies, matching `:3412–3465`.
The existing `processed_color_change/inactive-tool` public E2E stays unchanged
and must become GREEN: its override legitimately changes nozzle count and
colours and supplies an8-entry matrix. Retain its prior matrix-size RED from
`fresh-d789-identified/workspace.log`. No new internal-value assertions.

Dependency inspection reads support flags with the source's `get_at` semantics
(`:3473–3475`, `Config.hpp:624–627,1892`); it never broadcasts selectors or
datasets. If selector dependencies are unresolved, decline reconstruction before
entering its support-size/minimum checks. For admitted reconstruction retain the
source's explicit support/colour count equality check (`:3412–3418`). This keeps
the public API's supplied valid matrix intact when its extra inactive tool has
no complete selector context; no padding of flag or selector vectors is added.

## Review boundary correction: generated ids and calculator reachability

The generic generated-printer-id rewrite remains exclusively in spec
`703658c84`; FM must consume a complete materialized id map, not implement
any portion of that generator. `PrintConfig.cpp:9033` defines completeness
against the entire variant vector (not merely physical nozzle count), and
`:9040` chooses generation for an incomplete map. Therefore FM's dataset
admission must also require ids covering every variant. If a calculator cell
needs that missing capability, preserve the supplied matrix without rebuilding;
independent multiplier resizing and dataset-free source constants remain valid.

For provenance, the two removed generator-zero branches *do* have upstream
counterparts: absent `extruder_variant_list` returns zero at `:9012–9013`, and
no matching flattened index returns zero at `:9028`. They are deleted from FM
because their owner is the separately specified generic materialization slice,
not because source arithmetic should be redefined. FM neither synthesizes ids
nor substitutes zero for an unresolved map.

Calculator calls are reachable only after destructuring `Some(dataset)`.
The `None` branch fills only source-independent diagonals/support destinations,
admitted by the existing cell-dependency predicate; it never invokes a
calculator or substitutes a dataset. Do not replace production unwraps with
expect/unreachable or introduce a palette-count special case.

Failure inventory before implementation: ids covering nozzles but not all
variants must decline reconstruction; complete maps must retain exact selection
and output; diagonal/support-only matrices must still compute without ids;
multiplier prefix resizing must precede a dependency decline. Existing focused
whole-output E2Es and all x2d_flush_matrix witnesses remain unchanged.
The follow-up review requires a new full seeded v2 run after BROWSER-END:
compare every common case's raw Ares bytes against the previous 594-test
candidate, inventory and explain any changes, and require seeded all PASS.
Focused/source-equality evidence alone cannot prove this stricter guard inert.
Official need not rerun; retain its previously accepted evidence.

## Official-mode whole-file case disposition (coordinator ruling 2026-10-09)

The two official-control failures (`bridge_over_infill_multi_filament`,
`bridge_over_infill_multi_filament_x2d`, `narrowed_x2d_whole_file`) are
not byte divergences: both trees fail with `StageError::Timeout` — under
the official control set's 32-way parallelism the appimage wrote only
53KB of the 6.4MB stream before the shared 300s stage timeout killed it
(`/tmp/narrowed-x2d-whole-file-rqnfpn/orca/plate_1.gcode.tmp`, command
evidence `/tmp/narrowed-x2d-whole-file-rqnfpn/command-kCX0RF`, bridge
counterpart `/tmp/bridge-multi-i4pIxg/command-kKsHJu`). Standalone the
same fixtures complete quickly: narrowed X2D 4.2s unpinned / 12-13s
pinned to one CPU, K3M 0.6s (probes under `/tmp/x2d-official-probe/`).

The stock official appimage additionally labels objects from an
uninitialized id: three probes on the same input and CPU pin produced
`id:0` twice and `id:9223372037920129024` once (1424 label lines,
`/tmp/x2d-official-probe/{out,out1,out2}/plate_1.gcode`). The `id:0`
run is byte-identical to the seeded reference modulo the `; generated
by` banner (diff: 0 lines); the garbage-id run differs only in those
label lines. The seeded replay binary initializes ids (the
`initialized_object_id` precedent, `ARES_PARITY_OBJECT_ID_REFERENCE`),
which is why the seeded control set passes these cases byte-exact.

Disposition (option ii, not comparator normalization — normalization
would be reused by other cases and set a relaxation precedent):

- The two whole-file tests keep the byte comparison for the seeded
  reference only.
- Their native stage timeout extends to 1800s (test-local, not the
  shared default), citing the 53KB/300s contention evidence.
- Under official the cases assert the native completed and produced a
  complete stream (`; CONFIG_BLOCK_END`, `; total layer number:`).
- Under `ARES_REQUIRE_ORCA=1` a missing native must still FAIL (the
  runner `from_env` error path is unchanged).

This is tests-only; no production or comparator change.

## Closure record: exit-condition evidence (coordinator-approved plan, 2026-10-09)

Landed chain on main: RED `ab6d7d2e1` → GREEN `e0134bcd7` → review
fixes `f14c02af6`/`0c6e3ccf8` → dependency rule `310f5ae9a` → generated-id
reselection RED `1c3a64a59` → GREEN `cbee83539`. Roadmap exit conditions,
each with its verifiable evidence — gaps stated explicitly:

1. **Fresh same-input Native RED (hash).** Admitted RED `ab6d7d2e1`
   (`red/admitted-red.json`: base `9e3e81837`, spec tree `33f240f3`,
   seeded `case-LqoYic` + official `case-bMQWtU`, verified native-cache-v2
   receipts, input SHA `b980548a…`, reference SHA `b566726656…`).
   Qualification: that input is the scalar UB shape later reclassified by
   the O8 contract and retired from acceptance; the accepted
   fully-specified missing-matrix shape is exercised by the live control
   `x2d_flush_matrix_missing_matrix_rebuilds_without_nozzle_type_change`
   (test-lineage RED at `256d17990`, flipped GREEN by `e0134bcd7`).
2. **Prepared nozzle-major matrices exactly equal.** The live controls
   assert the full-line bytes against complete Native output:
   rebuild `0,102,186,0,0,339,621,0`; preserved custom
   `0,42,42,0,0,210,210,0` after one [0.3,1] scaling; multiplier
   prefix `0.3` + second-head default `1`. Whole-output PASS in the
   seeded run below. **High Flow end-to-end landed (2026-10-09):**
   RED `147948913` exposed that Ares errored
   `invalid Orca option filament_extruder_variant` where Native slices
   — the variant-miss policies were not ported. GREEN `3c8ba32e4`
   ports them: `update_values_to_printer_extruders` falls back to
   variant index 0 silently (PrintConfig.cpp:9677-9681) and the
   multiple-filaments overload rescans the id vector for the entry's
   own id (PrintConfig.cpp:9849-9863). The control
   `x2d_flush_matrix_high_flow_dataset_matches_native` (legal
   [High Flow,High Flow] + absent matrix) now compares complete output
   byte-exact: nozzle_flush_dataset=2,2, matrix
   0,102,186,0,0,339,621,0, activate_air_filtration_during_print=1,0
   via the id-position index.
3. **Complete G-code equality.** Seeded v2 full run at `310f5ae9a`:
   **594/594 PASS, zero skipped** (`review-e0134/final/seeded-v2.log`);
   focused 48/48 incl. all x2d cases. Raw Ares byte comparison against
   the previous 594-run candidate: **500/500 complete streams
   identical** (`final/previous-candidate-raw-byte-comparison.json`).
   Current main (`b3f8894fd`-lineage, 104599195-rebased tree): seeded
   full 830/830 PASS (2026-10-09) incl. the x2d family; x2d_flush_matrix
   11/11 after the O8 closure fixup.
4. **Nontrigger rejection and control bytes preserved.**
   `x2d_flush_matrix_no_trigger_keeps_malformed_matrix_rejected`
   (Native reject + exact Ares error, both live);
   `x2d_flush_matrix_unchanged_nozzle_types_preserve_custom_matrix`
   (custom matrix retained, scaled once); seven seeded template-RNG and
   BBL SYNC controls retained `flush_volumes_matrix=0,0` /
   multiplier `0.3,1` per the diagonal ruling (`fresh-d789-identified`,
   `rebase-1b175/prior-failure-inventory.json`).
5. **Reference/workspace/Tier1 gates.** At `310f5ae9a`: seeded 594/594,
   focused 48/48, strict clippy/fmt (`review-e0134/final/clippy.log`).
   The workspace/WASM/browser trio was recorded on the e0134-era
   candidate (`final-21c-ctx`, `final-b610/REVIEW.md`: workspace
   7969/7969, WASM32/release/web builds, browser 15 PASS) and is
   **covered by later main gates**: every post-e0134 production merge
   ran the full default workspace on a tree containing this slice —
   e.g. the by-object merge `5c9e69226` ran workspace 8203 passed /
   6 skipped on its 104599195-rebased tree
   (`${TMP_ROOT}/ares-w5-gates/ws-default-c10.log`, 2026-10-09) and the
   ares-wasm crate compiled clean on the same branch lineage
   (`${TMP_ROOT}/ares-w5-gates/wasm-check-final.log`, 2026-10-08); the web
   browser gate ran on later main per coordinator confirmation
   (2026-10-09). The `310f5ae9a` bridge is
   `final/source-equality.json` (only `flush_matrix.rs` changed).
6. **Coordinator exact-hash approval.** Spec admission: tree `33f240f3`
   (bound in `admitted-red.json`). GREEN review: exact candidate
   `310f5ae9a2c1ee510e89396335d876629eac86a4` per
   `review-e0134/REVIEW.md` (supersedes the e0134bcd7 request); the
   commit is on main. The coordinator's own approval ledger is outside
   this repository — the hashes above are the reviewable objects.

Boundary-table ↔ module checklist (static, 2026-10-09):
`OrcaSlicer.cpp:1708-1718,3022-3047` → `flush_matrix/context.rs`;
`:3340-3351` predicate, `:3412-3465` resize/dataset, `:3467-3503`
recompute → `options/flush_matrix.rs` (191 LOC); `FlushVolCalc.cpp` +
predictor/colour math + datasets → `options/flush_volume/` (273 LOC +
`data/flush_data_{standard,dual_standard,dual_highflow}.txt`);
`Plater.cpp:764-835` minima → `flush_matrix.rs::minimum_volumes`;
`PrintConfig.cpp:9004-9060` → `project_variants::get_index_for_extruder`;
`GCode.cpp:5596-5614` export scale/round → the unchanged
`config_export/transform.rs::scale_flush_matrix` (the temporary
one-palette matrix shell is gone from `prepare_multi_extruder_cli_defaults`,
which now only carries the `:5993-6022` AMS default). Seams verified in
`project/effective_config/phases.rs` (single preparation call before the
first normalize pass). Scalar UB inputs follow the closed O8 contract.

## Sequence, validation and exit

1. Submit this spec and wait for exact-hash coordinator approval.
2. On fresh current main prewrite the minimal admitted RED/control set; record
   fresh raw/seeded/official producer provenance and failing base evidence.
   Commit tests separately. Unknown-producer historical evidence stays labeled.
3. Implement only the source ledger/preparation seam, with no emitter patches.
   Require complete seeded output equality and expose official failures honestly.
4. After the coordinator's browser low-load window ends, run workspace
   `cargo nextest run --workspace`, strict workspace/all-target/all-feature
   Clippy, fmt, core+adapter WASM and current-base full seeded/official controls
   using `${TMP_ROOT}/ares-parity-control-set-v2.sh`, including
   `ARES_PARITY_PROFILE_ENTRY_REFERENCE=1`. Preserve complete comparisons,
   unchanged-stream inventory and raw/stable processor oracles where required.
   Browser E2E is sequential base/candidate, workers1/retries0, load<20, under
   an explicitly coordinated window; do not run heavy jobs during web's window.
5. GREEN commit stays separate from RED. Request sole coordinator exact-hash
   review with source citations, all-case artifacts and every retained failure.
   Fresh fetch/rebase and affected full gates precede nonforce FF integration;
   verify remote main and notify.

Exit requires preparation trigger correctness, exact recomputed values and
nozzle layout, source-prescribed nontrigger rejection, one export scaling,
full-output Native parity, unchanged neighboring controls and Tier1/API/lint
verification. Removing only the matrix error or matching only8values is not
completion. This milestone does not claim all dual-nozzle printers or the
entire upstream CLI rewrite are complete.
