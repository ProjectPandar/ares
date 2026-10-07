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
This is #55 / `X2D-FM` in `/tmp/ares-matrix-clusters.md`, independently owned by
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

Historical case `/tmp/ares-vg1-red-artifacts-matrix/case-Uut51r/` binds the
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

Preserve `/tmp/ares-x2d-flush-matrix/green/seeded-focused/case-9whGha/`
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
under `/tmp/ares-x2d-flush-matrix/fresh-d789-identified/candidate-seeded.log`
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
`/tmp/ares-x2d-flush-matrix/trace/compact-aa.native.log` has flush computation
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
   using `/tmp/ares-parity-control-set-v2.sh`, including
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
