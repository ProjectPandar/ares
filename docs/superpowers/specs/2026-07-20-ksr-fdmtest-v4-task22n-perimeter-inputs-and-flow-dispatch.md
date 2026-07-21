# Task 22N: Single-Region Perimeter Inputs and Flow Dispatch

## Status and objective

This specification is a draft. Production or tracked-test implementation may
begin only after this specification and its implementation plan are frozen as
one exact content frame and receive independent fixed-source/specification and
current-Ares/plan approval.

Task 22N is the next bounded source rewrite in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`df0652470fabfc7487ae13187cf0bc4a20bced99` produces the complete Task 22M
post-elephant-foot state. Task 22N ports OrcaSlicer's immediately adjacent
single-region perimeter-input preparation through the exact generator dispatch
decision. It consumes Task 22M state and typed effective Options resolved only
from the supplied 3MF.

The stage produces one immutable perimeter-generator input record for every
nonempty retained single-region layer. A record names the current region and
compatible-region set, lower and upper `lslices`, upper same-region slices,
layer identity and geometry parameters, four source-compatible `Flow` values,
spiral/model-rotation state, and the selected Classic or Arachne dispatch. An
empty region produces no generator record, matching the fixed caller's clear
and return behavior.

Task 22N deliberately stops immediately before entering
`PerimeterGenerator::process_classic()` or `process_arachne()`. It does not
claim to generate perimeter loops, extrusion entities, overhang segments, gap
fills, fill surfaces, or G-code. This is a real upstream call boundary, not an
Ares-owned pipeline stage: the wrapper exists only to retain the exact inputs
that the next fixed-source rewrite consumes. The public project API executes
Task 22N and continues to return `SliceError::ProjectSlicingIncomplete`.

## Fixed identities and source blobs

The fixed Ares baseline is commit
`df0652470fabfc7487ae13187cf0bc4a20bced99`, tree
`2fd30b34576755a884a9927c45f4432e70216dde`. Exact-SHA Tier-1 run
`29761944705` passed format, Ubuntu/Linux, Windows, macOS, and WASM/browser.

All upstream citations refer only to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored upstream checkout is
currently at a different commit and is not a semantic source. Audits, oracle
builds, and reviews must read fixed objects with `git show` or `git grep` at
the fixed commit. Tracked tests never inspect Git or the Orca checkout.

Fixed source blobs used by this slice are:

- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/PrintObject.cpp`,
  `925da0c5644e06b6813747ae35b371d1a1555fe1`;
- `src/libslic3r/Layer.cpp`,
  `5bdc156d0172ec19894b630cc70d73b5aef8f82d`;
- `src/libslic3r/Layer.hpp`,
  `cb2e6c7c1a166a028ac8fceffaf9f42f3c2426b0`;
- `src/libslic3r/LayerRegion.cpp`,
  `22e0a26898c6fe07ad8ebd35de303b5911d84f4b`;
- `src/libslic3r/PrintRegion.cpp`,
  `5c08de8b36d469b583425524c9948b92117236e8`;
- `src/libslic3r/Flow.cpp`,
  `42fd6e8ea132f8012217c38db7d3b7a36e2bbc76`;
- `src/libslic3r/Flow.hpp`,
  `79cb1b324d6343e41ed11a5f2984f52c0ea61412`;
- `src/libslic3r/PrintConfig.cpp`,
  `982953afa50af0217a4d64639116ff4a2e596e90`;
- `src/libslic3r/PerimeterGenerator.hpp`,
  `e4f918d8bd772e53b925dfdbcd57dc799261f2af`;
- `src/libslic3r/PerimeterGenerator.cpp`,
  `1a0f129c0d44cb5ff6c5b69ffee5ce5d211a0c80`;
- `src/libslic3r/Fill/FillBase.cpp`,
  `93586679821df5fe218b23e74ffc1723d2297bd5`;
- `src/libslic3r/Print.hpp`,
  `c69c5b6570a79cb750c08805e4907eeec5c834f5`;
- `src/libslic3r/PrintConfig.hpp`,
  `0a7b7ba36f87c3d4517daf96d7d8825812e66358`;
- `src/libslic3r/Config.hpp`,
  `5fedaa9b288e206b2dbf454927479c745d20e45d`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp`,
  `11e2d081d2455c1c969079501ea55348191de6db`; and
- `src/libslic3r/Feature/Interlocking/InterlockingGenerator.cpp`,
  `726ee2ffd827dd0226aa69ac74e5bac39d4aced8`.

## Predecessor and skipped slice-stage gates

Task 22M output is the only Task 22N geometry input. The complete M state,
including compensated `RegionSurface` values, uncompensated ordered `lslices`,
plans, volume sidecars, region identity, object occurrence identity, coordinate
scale, and effective typed config, is preserved exactly.

Before perimeter preparation, fixed `PrintObjectSlice.cpp:1208-1243` reaches
painted MMU segmentation, fuzzy segmentation, and interlocking. The committed
KSR archive proves these paths inactive through its loaded data, not through a
filename or fixture branch:

- its mesh contains no MMU/painted-face attributes and effective segmented
  width/depth values are zero;
- effective `fuzzy_skin` is disabled and the mesh has no painted fuzzy facets;
- effective `interlocking_beam` is false, so fixed
  `InterlockingGenerator.cpp:26-31` returns immediately; and
- it contains one normal model part, no modifier, and one retained region per
  layer.

Tracked real-archive tests must assert those facts through the public loader.
Activated painted/fuzzy/interlocking variants remain separate source-cited
slices. Task 22N must not silently discard activated data or add an identity
fallback.

## Exact upstream rewrite boundary

The owning call graph is:

1. Fixed `PrintObject.cpp:453-558`, `PrintObject::make_perimeters()`, invokes
   slicing and then visits every retained layer. The old extra-perimeter block
   at lines 480-542 is disabled by the fixed unconditional `continue` and is
   not ported.
2. Fixed `Layer.cpp:185-225`, `Layer::make_perimeters()`, clears empty regions
   and takes the one-region fast path at lines 222-225. Compatibility grouping,
   merged slicing, and output redistribution at lines 139-180 and 205-279 are
   deferred because Task 22M rejects valid nonempty multi-region input.
3. Fixed `LayerRegion.cpp:82-142` constructs the generator input, attaches
   lower/upper `lslices` and upper same-region slices, sets layer ID, resolves
   external, internal, overhang, and solid-infill Flow, and selects Arachne only
   when the object requests Arachne and spiral mode is false. Task 22N stops at
   the dispatch decision on lines 138-141 without calling either process body.
4. Fixed `PrintRegion.cpp:7-54` owns role-to-filament selection, first-layer
   width selection, role width selection, object-width fallback, nozzle
   selection, and `Flow::new_from_config_width` entry.
5. Fixed `PrintObject.cpp:3602-3661` applies scoped feature-filament overrides:
   a nonpositive raw override clears the explicit mask and may inherit a
   positive base `extruder`. Fixed `PrintObject.cpp:3562-3565,3694-3700` then
   clamps a final nonpositive or out-of-range selector to one, so the effective
   region config reaching this stage is positive and one-based. Fixed
   `Config.hpp:624-630` retains element-zero fallback for a direct out-of-range
   `get_at` request.
6. Fixed `LayerRegion.cpp:21-58` owns ordinary and bridging Flow entrypoints.
   Fixed `Flow.cpp:20-35,129-143,146-229` and
   `Flow.hpp:16-25,52-139` own auto width, f32 conversion, spacing,
   bridge-thread construction, cross-section adjustment, and volume per mm.
   Fixed `PrintConfig.cpp:10427-10430` rejects nonpositive `bridge_flow` before
   the reached generator path.
7. Fixed `PerimeterGenerator.hpp:73-141` defines the input contract retained by
   the Rust wrapper. Its process bodies and extrusion output types are outside
   this slice.

The Rust destination is a new crate-private `project_slice::perimeters`
module. It consumes `PostCompensationPrintObject` and owns a post-M wrapper plus
prepared records. The existing public/legacy `perimeters::generate_perimeters`
rectangle-based compatibility implementation remains exclusively on the old
STL pipeline. It is neither called nor used as a fallback by the project path.

## Prepared record semantics

For each object occurrence, preparation validates all records before consuming
or mutating Task 22M state. The output wrapper owns the unchanged M object and
one layer slot per planned layer. `PostCompensationPrintObject` gains an
always-compiled crate-private borrowed accessor so global preflight can inspect
every object before the complete vector is moved. Each nonempty slot contains:

- source object index, transform occurrence index, planned layer index and
  fixed layer ID;
- the sole region ID and a compatible-region list containing exactly that
  region;
- current `(region_index, layer_index)` ownership, optional lower/upper layer
  indices into the wrapper's ordered `lslices`, and optional upper
  same-region `(region_index, layer_index)` ownership;
- layer height and `slice_z`;
- `perimeter_flow`, `ext_perimeter_flow`, `overhang_flow`, and
  `solid_infill_flow`;
- exact spiral-mode and model-rotation values; and
- an exhaustive `Classic` or `Arachne` dispatch value.

The record uses indices into owned state rather than duplicating geometry in
production. Accessors used by tests and the next stage must resolve those
indices to the complete ordered `&[RegionSurface]` collection, never one
surface, and to the complete referenced `lslices`. Lower and upper are the
previous and next retained vector slots; upper same-region is region zero at
the upper slot for the accepted single-region boundary. Empty current surface
collections produce no record while the M state remains present. Zero-layer
objects produce empty record vectors.

Spiral mode is the fixed conjunction of global `spiral_mode`, layer ID at or
above `bottom_shell_layers`, and `print_z >= bottom_shell_thickness - EPSILON`.
Model rotation is zero unless `align_infill_direction_to_model` is active; when
active it is `atan2(m10, m00)` from the matching print-object occurrence
transform, with the same matrix orientation as the fixed caller. A new
crate-private `Transform3d` accessor returns the stored `(m00, m10)` pair
directly, including signed zero; production must not reconstruct that column by
transforming and subtracting points. Arachne plus spiral dispatches Classic;
there is no Arachne-to-Classic compatibility fallback beyond that explicit
fixed rule.

## Exact Flow semantics

The Rust `Flow` record stores width, height, spacing, nozzle diameter, bridge
flag, and source-compatible `mm3_per_mm`. Constructor arithmetic preserves each
fixed f32 narrowing and the C++ float-to-double promotions. `mm3_per_mm` first
narrows the fixed expression to f32 and then widens that result to f64. Flow
value equality follows fixed `Flow.hpp:83-86`: it compares width, height,
nozzle diameter, and bridge flag, while ignoring derived spacing and volume;
exact tests compare every field's bits separately. Task 22M's minimum-width
consumer remains the f32 sum `width + spacing`.

For ordinary external, internal, and solid-infill roles:

1. Select `initial_layer_line_width` on layer ID zero only when its raw value
   is positive; otherwise select the role-specific width.
2. External uses `outer_wall_line_width` and `outer_wall_filament_id`;
   internal uses `inner_wall_line_width` and `inner_wall_filament_id`; solid
   uses `internal_solid_infill_line_width` and
   `internal_solid_filament_id`.
3. A zero role width falls back to object `line_width`. A final non-percent
   width at or below zero uses `1.125f * nozzle_diameter` for all three reached
   roles. Percent widths resolve against the selected f32 nozzle.
4. Effective filament selectors reaching preparation are positive and one-based.
   A raw nonpositive scoped override first clears the explicit override and may
   inherit a positive base `extruder`; only a final nonpositive or out-of-range
   value is clamped to one. KSR fixes the base extruder at one, so its raw-zero
   role selectors become effective one and subtracting one selects nozzle
   element zero. The pure Flow seam separately preserves direct
   `ConfigOptionVector::get_at` fallback for synthetic zero, negative, and
   out-of-range selectors; those values are not claimed reachable through a
   valid public 3MF. No `filament_map` lookup occurs in either seam.
5. Height is the planned layer height converted to f32. Nonbridge spacing is
   `width - height * float(1 - 0.25 * PI)`. Nonbridge volume evaluates the
   promoted expression
   `height * (width - height * (1.0 - 0.25 * PI))`, narrows it to f32, and
   widens that result to f64. Bridge volume first performs `width * width` in
   f32, completes the circular-area expression in f64, narrows to f32, and
   widens to f64.

Overhang Flow uses the internal-perimeter role selector and nozzle. It resolves
`bridge_line_width` against that nozzle and reads `bridge_flow` plus object
`thick_bridges`:

- thick mode uses the configured positive bridge width or nozzle as thread
  diameter, multiplies by `sqrt(bridge_flow)` for the validated positive ratio,
  and constructs a circular bridge with width equal to height, spacing
  `diameter + 0.05`, and bridge flag true;
- nonthick mode starts from ordinary internal-perimeter Flow, optionally
  replaces width with the positive bridge width at the same height/nozzle, and
  applies `with_flow_ratio`; and
- `with_flow_ratio` narrows `mm3_per_mm() * ratio` to the f32 area argument and
  follows fixed `Flow::with_cross_section` branches and EPSILON comparisons
  rather than multiplying width directly. The reachable increase/grow-height
  branch retains spacing; the decrease/shrink-width branch calls `with_width`
  and therefore recomputes rounded-rectangle spacing; the decrease/round
  branch retains the prior spacing; and the tolerance branch returns the exact
  original Flow. The increase branch whose
  `new_full_spacing <= current_spacing` requires a prior noncanonical
  `with_spacing` mutation and is outside this preparation slice.

The stage rejects nonfinite or source-invalid externally supplied values before
state consumption and reports the owning Orca option key. In particular,
nonpositive `bridge_flow` is `invalid Orca option bridge_flow` before either
thick or nonthick construction; it is not a successful zero-cross-section Flow.
It does not add internal defensive copies or validate states already guaranteed
by the typed loader and Task 22M invariants.

## KSR reached branch inventory

The committed project fixture is 698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9.
Its reference G-code is 10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3.
The Task 22N real-archive test must load and observe, among other values:

- Classic generator, two walls, nonspiral mode, and one region;
- initial/external/internal/solid widths 0.5/0.42/0.45/0.42 mm;
- nozzle diameters `[0.4, 0.4]`, raw role selectors zero, effective normalized
  selectors one, and resulting selection of nozzle element zero;
- bridge width 100 percent, bridge ratio 1, and nonthick bridges;
- model-aligned infill disabled; and
- 460 retained planned layers with exact lower/upper adjacency.

The later Classic implementation must also honor reached precise spacing,
dynamic top-one-wall, overhang splitting, smaller-width external loops, and
perimeter gap fill. Task 22N records their typed Options only as predecessor
state; it does not consume or claim them.

The old master plan text saying `gap_fill_target=nowhere` suppresses perimeter
gap entities is superseded. Fixed `PerimeterGenerator.cpp:1192,1325-1332,
1573-1624` enables perimeter gap generation from `gap_infill_speed > 0`.
`gap_fill_target` is read later by `Fill/FillBase.cpp:193-203`. The KSR archive
has `gap_infill_speed=250` and its reference has 470 Gap infill feature blocks.
No current or future project-path slice may use `gap_fill_target=nowhere` to
suppress those perimeter gaps.

## Independent oracle and checkpoint contract

Before GREEN, an ignored fixed-source C++ oracle must freeze exact N-specific
Flow and context payloads for synthetic cases and the committed KSR archive. It
may reuse fixed Orca headers or transcribe the cited formulas, but it must
compile from fixed objects, use `/fp:precise`, and never call Ares. For KSR it
accepts only the approved independent Task 22M oracle output, exactly 3,008,346
bytes / `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`,
and preserves those bytes rather than re-encoding them.

An independent ZIP/JSON/XML probe freezes raw KSR Options, overrides,
normalization, and transform. The ignored oracle-only KSR profile reconstructs
missing planned-layer scalars with the fixed additive recurrence `lo = 0`,
`hi = 0.2`, then per layer `height = hi - lo`, `print_z = hi`,
`slice_z = 0.5 * (lo + hi)`, followed by `lo = hi; hi += 0.2`. It must not use
`(index + 1) * 0.2`. These values are evidence only; production and tracked
archive tests always use the real loader.

The oracle runs twice from a clean build and produces byte-identical output. A
deliberately wrong pre-mapped-nozzle mutant and direct-width bridge-ratio mutant
must differ. Its independence claim covers the new N payload; the approved M
wire is a predecessor artifact, not behavior recomputed by Task 22N.

Tracked Rust expected values are copied from the approved oracle before the
production boundary exists. Tracked tests never execute the oracle, inspect
Orca/Git, read the reference G-code, or pin source layout. Synthetic coverage
includes first/later layers, every role, absolute/percent/auto/fallback widths,
selector zero/one/two/out-of-range, two nozzle diameters, thick/nonthick bridge
ratios across every preparation-reachable cross-section branch, empty/nonempty
layers, multiple surfaces in current and upper collections, adjacency, spiral
dispatch, model rotation, multiple object occurrences, transaction failure,
and exact f32/f64 bits. Zero/negative bridge ratio cases are preflight errors,
not successful records.

The test-only checkpoint begins with `ARES22N\0`, writes a u64 predecessor
length and the exact complete `ARES22M\0` wire, then encodes the independent
N-specific payload for every layer slot. A present record includes identity,
compatible region IDs, complete current/lower/upper/upper-same-region resolved
geometry, layer scalars, all raw Flow fields plus volume per mm,
spiral/rotation, and dispatch. Parsers reject wrong magic, truncation, malformed
counts and enums, noncanonical booleans, and trailing bytes, and require exact
EOF.

The M wire intentionally lacks coordinate scale, layer scalars, typed config,
and non-kind Surface metadata. Checkpoint equality therefore proves exact M
wire identity, not complete in-memory-state preservation. Tracked Rust
structural tests separately prove that the wrapper preserves scale, plans,
sidecars, full Surface metadata, resolved config, and owned geometry.

Real in-memory 3MF archive pairs vary one Option at a time while preserving
all other semantic entries. At minimum they cover each role width, initial
width, object fallback, each role selector, nozzle list, bridge width/ratio,
thick bridges, spiral/bottom-shell gate, alignment/transform, and generator
selection. Each pair proves identical M input when the changed Option is not
consumed before N and the independently predicted N-only change. Flow Option
pairs set elephant-foot compensation to zero so Task 22M does not consume their
width/nozzle values. Selector archives use effective one versus two. A raw-zero
versus raw-one pair with base extruder fixed at one proves the same effective
one, while a raw-zero/base-two versus explicit-two pair proves scoped fallback
to the same effective two. Alignment pairs hold one transform fixed and vary
only the alignment Option. Any pair whose Option necessarily changes an earlier
stage freezes the expected M delta instead of falsely requiring M equality. A
dedicated anti-map pair swaps `filament_map` and must not change any Flow. Each
archive pair maps to an already frozen synthetic oracle case; the ignored C++
oracle does not parse or derive expected values from every tracked test
archive.

## Native, WASM, browser, and public lifecycle

Task 22N replaces the non-default `task22m-browser-oracle` feature with
`task22n-browser-oracle`; no alias remains. The feature build exports exactly
`task22nBrowserInputOracle` and `task22nBrowserOracle`. Default core and WASM
builds expose no Task 22 checkpoint hook.

Browser tests run parser KATs before fixture fetch, build Option-only archives
with `fflate`, compare complete native-registered N frames, verify predecessor
M identity and exact EOF, and execute twice in fresh optimized bindgen output.
Public `slice_project` must traverse N and return
`ProjectSlicingIncomplete` for the KSR fixture on native and browser. N bytes
are never returned by the public slicing API and are not G-code.

## Source structure and exclusions

Production code is split into real Rust modules under
`project_slice/perimeters/`; tests live under
`project_slice/tests/perimeters/`. No changed Rust source or test file may
reach 400 physical lines. Source splitting may not use `include!`,
`include_str!`, `include_bytes!`, path indirection, generated source text, or a
new broad lint allowance. Fixture-byte `include_bytes!` in an ordinary test is
permitted only as data, never to split Rust source.

No new unsafe code, dependency, filesystem access, terminal behavior, native
thread assumption, fixture/hash/name branch, reference-output read, hardcoded
KSR geometry, old rectangular perimeter call, gap proxy, Arachne fallback, or
legacy output adapter is allowed. The implementation stays portable across
WASM, Windows, macOS, and Linux.

## Acceptance and review loop

The implementation is complete only when focused Flow/context/checkpoint tests,
all Task 22 predecessor tests, full `ares-core`, workspace nextest, formatting,
strict clippy, all-target checks, default and feature WASM, two real Chromium
runs, structural audits, fixture identity, and repeated N output identity pass.
Architecture and roadmap documents must record that N prepares inputs only,
that public slicing remains incomplete, and that the next boundary is the full
KSR-reached Classic process.

One dedicated independent read-only reviewer then validates the frozen exact
frame in six sections: requirement completeness, fixed-source logic, boundary
cases, code quality, test/oracle coverage, and actual native/WASM/browser
execution. It returns P0-P3 findings and a repair checklist without editing.
The main thread repairs every finding, reruns affected and complete gates,
freezes a new frame, and returns it to the same reviewer. This repeats until
all six lists are empty and the verdict is APPROVE.

Only the unchanged approved frame may be committed and pushed. Local HEAD,
upstream tracking ref, and direct remote readback must agree, and the exact-SHA
Tier-1 run must pass format, Ubuntu, Windows, macOS, and WASM/browser before
Task 22O begins.

## Six-axis review repair amendment (2026-07-21)

This amendment supersedes only the earlier claim at lines 252-254 that the
increase branch with `new_full_spacing <= current_spacing` requires a prior
noncanonical `with_spacing` mutation. The dedicated six-axis review found that
claim false for metadata-valid canonical Flow. All other approved Task 22N
scope, source identity, KSR identities, deferrals, and release gates remain
unchanged.

### Complete fixed `with_cross_section` branch

Fixed `Flow.cpp:167-197` has two sub-branches when
`area_new > area + EPSILON`. If f32 `area_new / height` is greater than the
current spacing, it grows height while retaining spacing as already specified.
Otherwise it calls `with_width` with the rounded-rectangle width reconstructed
from f32 `area / height`, and `with_width` recomputes canonical spacing. The
second sub-branch is reachable without any prior `with_spacing` call and must
be ported; it may not be replaced by an assertion.

The fixed-source exact reducer is nonthick overhang Flow with height
`0x4113a9f3` (`9.2289915`), nozzle `0x4253561c` (`52.83409`), canonical width
`0x440415d2` from `1000%`, spacing `0x44039711`, area `0x4597ce34`, zero
`bridge_line_width`, and `bridge_flow=1.0000001`. Its f32 `area_new` is
`0x4597ce35`, while `area_new / height` rounds back to spacing `0x44039711`.
Fixed Orca returns width `0x440415d1`, height `0x4113a9f3`, spacing
`0x44039710`, nozzle `0x4253561c`, bridge false, volume f32 `0x4597ce33`, and
stored f64 volume bits `0x40b2f9c660000000`.

A real archive reducer uses a synthetic closed model whose top Z values are
`18.5`; layer and first-layer heights `9.2289915`; both nozzles `52.83409`;
initial, object, inner, outer, and internal-solid widths `1000%`;
`bridge_line_width=0`; `thick_bridges=false`; and a unique `bridge_flow` delta
from `1` to `1.0000001`. Both sides retain the same M bytes. The repaired N
has two populated slots and only the overhang Flow changes to the exact bits
above. Public `slice_project` remains `ProjectSlicingIncomplete` and must not
panic or produce a WASM trap.

### Task 22M construction versus Task 22N record validity

The shared ordinary Flow constructor is also used by the already shipped
Task 22M elephant-foot preflight. It owns fixed spacing construction only; it
must not eagerly reject a finite positive width and spacing merely because the
stored f32 volume underflows to zero. A metadata-valid predecessor reducer with
nozzle `0.4`, layer and first-layer height `1e-30`, absolute initial width
`1e-30`, elephant-foot compensation `0.15` for one layer, and a closed prism of
Z height `1e-30` must preserve Task 22M success. Its width and height are
`0x0da24260`, spacing is the finite positive `0x0d7ee054`, and compensation
minimum width is `0x0e10d945`.

Task 22N is the boundary that stores complete Flow records. After resolving
each role and before consuming M state, it must reject any nonfinite or
nonpositive final `mm3_per_mm`. The predecessor reducer above therefore stays
successful through M but fails N with the existing
`InvalidInput("invalid external perimeter flow volume")` contract.

Fixed metadata accepts positive `bridge_flow` through 2 and fixed runtime
validation rejects only values `<=0`. Consequently `f64::MIN_POSITIVE` is a
valid external ratio even though its thick and nonthick derived f32 overhang
cross sections underflow to zero. Both modes must fail at the Task 22N Flow
record boundary, before M state consumption, as
`InvalidInput("invalid Orca option bridge_flow")`. This final-volume check
belongs to the overhang/N boundary, not the pure `with_cross_section` helper or
the Task 22M shared constructor.

### Repair evidence and structure

The fixed C++ `/fp:precise` oracle must cover the canonical increase-else
result and the fixed `mm3_per_mm()` rejection for the tiny-positive overhang
cases. Rust tests must cover the pure exact bits, Task 22M spacing-only result,
Task 22N transactional errors, a real archive/public reducer, and the generated
WASM/browser reducer. The tracked synthetic oracle and browser expectations
must be regenerated from the independent fixed-source probe where their
covered records change; the real KSR fixture, M, and N identities must remain
unchanged.

All four new `#[rustfmt::skip]` attributes in
`project_slice/tests/perimeters/flow.rs` are prohibited. That file must use
ordinary rustfmt-stable test data and remain below 400 physical lines without
adding a source-splitting macro. Task 22N role/oracle names must spell out role,
nozzle, and percent meaning; opaque `P/E/O/S`, `P4/E4`, and
`PP6/PE6`-style symbols are not permitted in the repaired Rust or browser
case tables.

The existing path
`crates/ares-core/src/project_slice/tests/compensation/fixture.rs` is added to
the planned path manifest solely for the real-3MF Task 22M predecessor
regression. No other new tracked path is authorized. Implementation may begin
only after two independent read-only reviewers approve the exact amended spec
and plan frame.

## Second six-axis review repair amendment (2026-07-21)

The repaired frame was rejected because fixed release behavior still had one
unmodeled decrease-rounding case and the approved plan retained a superseded
synthetic-oracle identity. This amendment changes no KSR-normal Flow bits,
Option inventory, public completion boundary, or Task 22O deferral.

### Fixed release decrease-rounding behavior

Fixed `Flow.cpp:167-197` asserts that the intermediate decrease width is
positive only through `assert()`. The independent oracle is compiled with
`/DNDEBUG`, so release behavior removes that assertion. A metadata-valid
nonthick overhang reducer reaches this distinction with nozzle `100`, initial
and inner-wall width `500%`, layer and first-layer height `2e-7`,
`bridge_line_width=0`, and `bridge_flow=f64::MIN_POSITIVE`. Fixed width
validation accepts exactly five nozzle diameters, layer-height metadata accepts
positive `2e-7`, and bridge-flow validation accepts a positive ratio.

The ordinary f32 Flow enters the decrease branch with width `0x43fa0000`,
height `0x3456bf95`, area `0x38d1b718`, and `area_new=0`. Subtracting the
decrease produces width `0xb8000000`; fixed release execution continues to a
zero-diameter Flow and rejects it when volume is consumed. Ares must not panic
or trap. It must let the zero-volume result reach the existing Task 22N
overhang boundary and return
`InvalidInput("invalid Orca option bridge_flow")` transactionally before M
state consumption. No new validation or fallback belongs inside the trusted
private geometry helper.

The fixed C++ probe must remove its hand-written positive-intermediate-width
throw so `/DNDEBUG` controls this behavior and add the reducer as an independent
error self-check. The synthetic wire contains successful Flow records only, so
this expected-error reducer must not be encoded as an aggregate object and the
wire schema must not be expanded to represent errors. Regenerating the tracked
25-object success aggregate must remain byte-identical at 23,747 bytes / SHA-256
`82ccfa1db8bcfea1c4689147561be8c7058c6fdefe0df9b7b8ad127e99487fd1`.
Pure Rust, real 3MF/public Rust, and generated real-archive WASM/browser
regressions must prove the exact error and absence of panic/trap. Every reducer
value must come from its in-memory 3MF; production code may not recognize the
fixture or hardcode its result.

The pure regression is authorized in the new focused module
`crates/ares-core/src/project_slice/tests/perimeters/flow_edges.rs`, because the
existing `flow.rs` is already at the 399-LOC ceiling. The browser reducer data
is authorized in
`crates/ares-wasm/tests/browser/task22n-edge-vectors.mjs` for the same reason;
the existing vector module is already 399 LOC. Both new source files must stay
below 400 physical lines and must be declared/imported normally, never through
an include macro. These two paths supersede the preceding no-new-path sentence
only for this repair.

### Oracle identity and review gate

The earlier 23,071-byte / `6cba4f...96bd` aggregate is historical and
superseded. The first repair's 23,747-byte / `82ccfa...87fd` aggregate is the
current success-record identity and must remain unchanged because the new case
is expected to fail before encoding. After the independent probe is GREEN, this
full identity and 25-object count must be reconfirmed in the plan, Rust parser
test, oracle README, architecture record, and roadmap. The spec/plan exact frame
must then receive two fresh independent read-only approvals before it is
returned to the same six-axis reviewer.
