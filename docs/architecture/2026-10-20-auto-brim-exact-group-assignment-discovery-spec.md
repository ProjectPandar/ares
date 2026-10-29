# Draft SPEC: exact-containment assignment of AutoBrim layer polygons

Status: **NATIVE-FIRST DISCOVERY / SPEC-REVIEW-REQUEST**; no RED or production
change authorized by this document. Branch `worker2/auto-brim-material-discovery`
(name reflects the initial exploration, not the final selected boundary).
Base and read-only CLI source: `e81afae89a4874af3c5556a592c3d5a1d121055c`,
the merged AutoBrim/admission/object-speed integration.
This slice rewrites one `libslic3r::PrintObjectSlice` assignment operation,
not filling, ironing, seam, Voronoi or a new Ares pipeline feature.

## Discovery result

A real merged exported-project input has one tall PLA cube with active AutoBrim.
Adding a second disjoint identical model-part volume to **the same object**
keeps Native brim active around both islands but removes all Ares brim motion.
Two coincident volumes instead match, and the exact disjoint project with
`no_brim` also matches. All inputs are one filament/extruder1; material-table
or cross-material adhesion behavior is unnecessary to reproduce this failure.

The initial two-material exploration also showed the failure, but its uniform
PLA control failed too. It was therefore rejected as evidence for a material
slice, reduced to one filament, and localized to exact polygon containment
assignment. No general mixed-material behavior is claimed here.

## Native-first evidence (completed, not public RED)

Final root:
`/home/indexyz/ares-coord/evidence/worker2-material-discovery-20261020T194111/final-single-filament-regroup/`.

Source project is the merged
`tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf`, SHA256
`e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5`.
It derives from a real exported cube project; mesh is10×10×2 mm with build
Zscale50, hence one10×10×100 mm model. Mesh vertices/faces are not changed.
The source object's one printable instance remains at110/110. No supports,
raft, tower, skirt or combined brims; PLA only, extruder1, ByLayer. The already
merged object-speed global settings remain the same for every case, with no
object/volume speed keys.

For paired two-volume inputs, copy the same mesh as a second model resource
(id3), add one component and matching `normal_part` metadata to object2,
extruder1. Only its X shift differs:0 for overlap vs30 mm for separated. The
original cube mesh entry is byte-identical to the source project in all4 inputs;
the second entry differs only in model identity/UUID, not mesh vertices/faces.
The no-brim variant differs from separated-auto only in `brim_type`.

All8 Native runs (4×official/seeded) and4 existing CLI runs exit0, serial CPU13.
Official/seeded Brim body hashes agree. Counts below are actual positive-E XY
extrusion lines in Brim sections, **not** inferred loop or TYPE counts:

| Input | SHA256 | Native / Ares positive Brim XY moves | Complete CLI output vs Native |
|---|---|---|---|
| single-auto | `e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5` | 448 /448 | equal, both modes |
| overlap-auto | `4b91d9a7884320b86ed44a0c112811aa90a5feb9e7f6381e40c3420b930d4a78` | 448 /448 | equal, both modes |
| separated-auto | `8ce90e912e8fb20f3c190a2b4b7971d8ba49b1b6cd8f9f1fdc9477e5e5541096` | 894 /0 | **different, both modes** |
| separated-none | `5b9ac2700a7365dea7b0a87aa46f6b9aeef447511de8ad1fea26a8a467bb7b02` | 0 /0 | equal, both modes |

Whole-output discovery comparison replaces only the generated-by line. This
is exploratory CLI evidence, not a committed API/CLI nextest RED or a new
comparator. API was not run. For separated-auto the first normalized byte
difference454 is Native `M73 P0 R131` vs Ares `M73 P0 R130`; all894 Native Brim
XY moves are absent in Ares. Do **not** call this the old independent M73 gap:
the exact same separated no-brim input matches completely, as do both active
single/overlap Auto controls. Still, only a future production GREEN with full
bytes equal can prove no estimator residual remains; never mask M73 to pass.

Parent evidence (mixed-material or array-expanded preliminary variants) is
not the final witness set and is not a proposed RED. Final observations are
recorded in `summary.json`, `positive-motion.json` and `verification.txt`.

## Upstream owner and source-local discrepancy

Upstream files read from local checkout commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, no edits:

- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp:628-675::reGroupingLayerPolygons`.
  Copy group slices and layer expolygons and simplify both with the configured
  resolution. Initialize every polygon's group index to **-1**, skip only a
  simplified layer polygon with nonpositive own area, and start its
  `minArea` at that polygon's area. For each group, sum the **signed** areas of
  `diff_ex(layer_polygon, group.slices)`. If `area < minArea`, record that group
  and area. A difference area of **zero is a valid best match**; strict `<`
  preserves the first winner on ties. Clear temporary group slices and append
  the original unsimplified polygons only for nonnegative assigned indices.
  `tbb::parallel_for` partitions layer-polygon indices: each iteration reads
  immutable `epsc`/`gvssc` and writes only its own `epsIndex[ie]`. There is no
  cross-index reduction or mutation of group slices inside the parallel loop.
  Group candidates are still visited in vector order for each index, and the
  post-loop append traverses original polygon indices in order. Scheduling
  cannot change ties or append order; a serial Rust loop preserves semantics.
- `PrintObjectSlice.cpp:798-807` invokes the volume-group pipeline with scaled
  print resolution; preceding `:517-626` intersection grouping/negative-volume
  subtraction are separate already-existing steps, not redesigned here.
- `Brim.cpp:477-527` consumes each group's reassigned slices and its volume mesh
  height: `:500-501::configBrimWidthByVolumeGroups` followed by per-polygon
  bump/clamp/quantization. `:260-305` supplies moment-dependent group width.
  Wrong group membership changes moments and can suppress the whole footprint.

Current Rust gap:
`crates/ares-core/src/project_slice/gcode_emit/brim/groups.rs::assign_polygon`
(initially around170-202 at the base) starts with `Some((0, polygon_area))`,
uses absolute difference-area sum, and **continues for `area <=0`**. Thus an
exact zero difference does not assign the matching group. If no positive
strictly smaller difference is accepted, every island defaults to group0,
even when another group's exact containment matches it. Putting distant islands
in one group's moment calculation can drive its width below the omission gate.

This matches the observed shape dependence: a single group/overlap happens to
use group0 correctly; two disjoint groups expose the lost exact match. This
source diagnosis is not a claim that a temporary code edit already achieved
parity: no production probe, guard removal or test weakening was performed.

## Exact Rust rewrite destination and architectural decisions

- Port the cited `reGroupingLayerPolygons` assignment semantics in
  `brim/groups.rs::assign_polygon` and its call from `group_volume_slices`.
  Keep group order, configured simplification, unsimplified assigned output,
  signed area sums, unassigned state and strict-less-than tie semantics.
- Do not initialize an unproven fallback group, reject zero-difference matches,
  add positive-area normalization, or invent defensive error/guard behavior.
  The existing carrier is a temporary compatibility shell around upstream
  `groupedVolumeSlices`, not an independent grouping policy.
- Preserve preceding union-find, safety offsets, negative-volume subtraction,
  raw volume/slice carriers, AutoBrim width/moment/material/speed functions,
  and subsequent brim/skirt emission. No global bounding-box workaround.
- Keep public core API, no filesystem/UI/native runtime added; WASM and all
  Tier1 platforms remain required. Each touched file<=400LOC.

Included: assigning actual first-layer expolygons to existing upstream-derived
volume groups, including exact containment, zero-difference selection, strict
minimal-area tie order and no-match exclusion. The source guard for layer
polygons with own area<=0 remains; it must not be confused with zero **difference**.
Deferred: object-wide multi-material adhesion, support/raft/tower, fill behavior,
new volume intersection/offset algorithms, negative-volume slicing semantics,
repeated instances, combined-brim ownership, new speed lookup behavior and
M73 estimator changes. No new Unsupported guard or fallback allowed.

## Frozen RED input provenance

After SPEC approval the four final discovery archives are copied byte-for-byte
into `tests/parity/auto-brim-exact-group-assignment/{single-auto,overlap-auto,separated-auto,separated-none}.3mf`.
The table above is the frozen SHA256 manifest; every input has one public test.
The external generator is `generate-inputs.js` under the final evidence root,
with its source SHA256 recorded there. It reads the already-merged single-auto
project, creates a second component/part with consistent transforms, copies
its mesh without vertex/face edits, and writes a deterministic stored ZIP with
per-entry CRC32. It is a discovery/reproduction script, not a committed test
probe. No output G-code is frozen in Git. This corresponds to GUI Add Part
plus zero/30 mm placement, not malformed project or a new unsupported setting.

## Proposed post-approval SPEC → RED → GREEN

1. Rebase this docs-only SPEC onto current main, reconfirm exact4 input hashes
   and Native observations before tests. Approval required; no implementation yet.
2. RED freezes these4 input-only cases, all referenced. Reuse the integrated
   `runner/native_api_cli.rs` (or the canonical shared helper selected by the
   coordinator), Native through required-oracle setup. Assert Native rc0 and
   nonempty Brim XY motion for Auto inputs, no Brim for none. Require Native/API/
   CLI strict complete byte parity and API/CLI raw equality; target must FAIL,
   all3 controls must PASS in official and seeded modes on assigned CPU13.
   Record the missing894 motion lines, not only early M73 differences.
3. GREEN changes **production only** at cited assignment boundary, preserving
   tests/docs and all width/speed/material/emission algorithms. If whole-output
   parity still fails, STOP and report the first residual; no M73 normalization,
   magic result constants, broad geometry cleanup or extra policy guard.
4. Existing AutoBrim volume/material/small-diagonal, per-object admission and
   object-speed regressions remain unchanged. After directed dual-mode GREEN:
   fmt, clippy, wasm32; exact-hash dual canonical FULL only in an explicitly
   coordinator-allocated window. Discovery runs no FULL. Review/merge only by
   coordinator; source-cited rewrite scope, no dead fixtures or Native outputs.

## Provenance / replay

Evidence scripts and inputs live outside Git: `generate-inputs.js`, `run.sh`,
`summarize.js`, exact input archives, Native command stdout/stderr/result/G-code,
per-case exit files, CLI outputs, input manifest, file/binary hashes and clean
pre-doc worktree status. All runs/analysis serial `taskset -c13`, seeded
`ORCA_REPLAY_CPU=13`, seed0; official extracted2.4.2 and explicit seed-only
Native binaries selected via existing wrappers. No builds, FULL, stress or
CPU48–51 use occurred.
Read-only Ares source `e81afae89`, binary SHA256
`543cae94560feb1f2a2ca4fad6603feb1406db637558895e17b823125838de4e`;
it was produced by the just-merged integration. Upstream source checkout is
read from the original worktree because this discovery worktree has no local
ignored OrcaSlicer source/resources link. Hashes are recorded in evidence.

Replay into a new external root: copy the3 scripts; verify source fixture and
CLI hashes; run `taskset -c13 node <root>/generate-inputs.js <root>/inputs`, then
`taskset -c13 bash <root>/run.sh <root>`. The generator copies the retained
real mesh and uses matching component/part transforms; no generated meshes or
probe code enter the repository. Only this SPEC document is committed.
