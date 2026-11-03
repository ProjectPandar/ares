# Draft SPEC: AutoBrim footprints in per-object skirt groups

Status: **DISCOVERY / SPEC DRAFT — no implementation authorization**.
Discovery branch: `worker2/next-skirt-brim-discovery-spec`.
Docs worktree base: `a5eb0f0356057fe9c62ec3319834f82b40a7731a`.
Dependency: reviewed, queued AutoBrim chain ending at
`c5ea87946cb8a0a08d483f158b89f2ff10e60228`; do not modify that chain here.
This is a bounded `libslic3r` rewrite slice, not an Ares-designed pipeline.

## Architectural decisions and next boundary

- Consume the existing AutoBrim producer only after its reviewed chain lands.
  Port `Print::_make_skirt`'s consumption of per-object AutoBrim occupied areas,
  grouping, and independent brim ownership, not another auto-width formula.
- Include ByLayer, `combine_brims=0`, single PLA filament, one instance per
  PrintObject, two positive skirt loops, `skirt_height=2`, `min_skirt_length=0`,
  uniform skirt angle, no support/raft/tower/draft shield, no local speed keys.
- Cover separated skirt groups and a pair where **AutoBrim occupancy alone**
  changes two skirt groups into one. Keep each object's uncombined brim owner.
- Do not add an Unsupported guard, fallback to combined skirts, or policy flag.
  Narrowing the existing type exclusion is a hypothesis, not a proven GREEN.
- Keep core platform-neutral and WASM-compatible. No core file I/O or UI.
- Public Native/API/CLI whole-output tests precede production edits; GREEN must
  not modify tests/docs. Existing assertions/comparator stay unchanged.

## Remaining upstream differences: discovery inventory

| Candidate boundary | Upstream owner | Current Rust gap / evidence | Disposition |
|---|---|---|---|
| AutoBrim per-object skirt occupancy/admission | `Print.cpp:2794-2819,2847-2896,2898-2917,2972-2990`; `GCode.cpp:4465-4540,5334-5335` | `skirt/generate.rs` excludes AutoBrim when an active plan exists; Native accepts and consumes its footprint (below) | **Selected next slice** |
| Contact-connected combined brims within a skirt group | `Print.cpp:2910-2970` (`brim_contact_distance`, union-find, cleanup, plate-coordinate infill) | `combine_brims` still rejects active multi-object perobject plans; `brim/combined.rs::enabled` deliberately excludes the perobject case | Source-only follow-up; no active paired Native proof collected, not an admitted RED |
| Object-wide material adhesion | `Brim.cpp:120-148::getadhesionCoeff` | Queued `brim/auto.rs` still uses group-filtered material carrier; first-layer extruding external-perimeter eligibility and last matching object volume are deferred in its accepted SPEC′ | Preserve deferral; requires different multi-material witnesses |
| Object-key-aware max speed | `Model.cpp:2976-3010,3222-3255` | Queued AutoBrim's resolved-region speed carrier does not represent object key presence/global wall-speed ratio semantics | Preserve deferral; not part of skirt admission |
| Sequential shared skirts | `Print.cpp:2552-2556` | Separate existing ByObject shared-group guard reflects Native rejection | Not a missing admission to delete |

The first row has independent Native geometry/role observations; the other rows
are an inventory, not parity claims. Prior historical AutoBrim specs deferred
admission because their then-probed Native inputs had no brim. The queued
geometry producer plus the active observations below supersede that premise,
not the explicit material/speed deferrals.

## Exact upstream and Rust destination

Source paths are under `OrcaSlicer/src/libslic3r`:

1. `Brim.cpp:260-305,448-530`: existing AutoBrim group width, per-polygon bump,
   clamp and quantization produce the footprint. The queued chain owns this;
   this proposal neither recalculates nor broadens its supported inputs.
2. `Print.cpp:2794-2819`: copy each object's hull across its instances and append
   the object's **plate-coordinate brim area contours** to occupied points.
   The selected one-instance inputs avoid an unproven repeated-copy producer.
3. `Print.cpp:2847-2896`: fixed-point skirt-envelope merging on occupied points;
   merging may be caused by brim occupancy, without brim-to-brim contact.
4. `Print.cpp:2898-2917`: with combine disabled, preserve singleton brim owners
   even when several objects share one skirt group. `2972-2990` installs those
   group brims with group skirt/member identities.
5. `GCode.cpp:4465-4484,4486-4540,5334-5335`: skirt once before the first group
   member, then each matching object's brim on its first layer, followed by that
   object's ordinary extrusion. A shared skirt does not pre-emit all brims.

Rust paths under `crates/ares-core/src/project_slice/gcode_emit`:

- `skirt/generate.rs::SkirtPlan::generate`: after queued AutoBrim, the existing
  `plan.is_some()` type whitelist still omits `ProcessBrimType::AutoBrim`.
  Preserve `combine_brims` and the independent sequential/shared-group branch.
- `skirt/occupied.rs::items`: audit existing `covered_hull`/`occupied_origin`
  against the cited plate-area frame. No double instance shift or extra skirt
  expansion; do not replace area contours with extrusion-path bounds.
- `skirt/member_groups.rs` and `skirt/loop_generation.rs`: reuse source-owned
  membership and fixed-point grouping; no duplicate grouping implementation.
- `layers/object_skirt.rs::emit` and `layers/brim_emit.rs::emit`: validate group
  once-emission and independent first-layer brim ownership/order. Flat skirt
  mirrors and current BrimPlan carriers remain temporary compatibility shells
  around the upstream objects, not owners of an independent pipeline design.

No production seam was modified or temporarily unguarded during discovery.
If strict parity needs more than the cited admission/consumer boundary, stop
and report the first raw difference before expanding this SPEC.

## Native-first experiments (completed, not committed RED tests)

Final evidence root:
`${COORD_ROOT}/evidence/worker2-next-skirt-brim-discovery-20261020T134127/final-no-support-no-tower/`.

Inputs derive from queued `auto-combined.3mf` (SHA256
`a8d9731e66be1021eca82d50813fdfde685c6af70df72656c0f6119b6ef7e74c`).
Mesh unchanged: two separately identified overhang objects, XY scale0.1/Z scale5.
Far instance X=80/140; near X=80/88; Y=110. Settings explicitly disable support
and prime tower, use `combine_brims=0`, width10/gap0, skirt distance2, loops2,
height2, ByLayer, one PLA. Variant changes only skirt/brim type and near X.
The prior exploratory `gap8/` and parent runs are not the final input set.

All twelve final Native processes (six inputs × official/seeded) exited0.
Both modes have identical role counts, positive-extrusion XY move counts and
bounds. Counts below are **first-layer TYPE blocks, not loop counts**:

| Input | SHA256 | First-layer skirt / brim blocks | All-layer positive XY skirt / brim moves | Existing CLI |
|---|---|---|---|---|
| far combined-auto | `ee5a89805ca47d75c0268015f393c4a60f8c278ab761b25a94ca1f88acd9a4a1` | 1 / 2 | 39 / 38 | exit0 |
| far perobject-auto | `339a797d71afc111b8857aae4db0ef405eb6a1534f1404a9a6762395651d4390` | 2 / 2 | 80 / 36 | exit1, existing skirt guard |
| far perobject-none | `b64e6d245c5df8e92a12df954f0f7caa1939ff1ee6235c58a2374b770cf7024e` | 2 / 0 | 66 / 0 | exit0 |
| near combined-auto | `686575c34b793f4f872f12737de583e8a1f579b025b54dfbca1db621b57c5736` | 1 / 2 | 47 / 37 | exit0 |
| near perobject-auto | `33879016885adcc85533e345a52a46d6ef4fbe74dd63156305f07bc1981d2d6a` | 1 / 2 | 47 / 37 | exit1, existing skirt guard |
| near perobject-none | `e22bfef922c343c7ba2e91514191c851ee9dd8ee89f11cd9e23cdf7e42e3baea` | 2 / 0 | 66 / 0 | exit0 |

The active perobject inputs fail CLI with
`UnsupportedProjectFeature("skirt_type per-object")`, not an environment error.
The other four controls match both Native files in the exploratory comparison
that replaces only the generated-by line. This is **not** a public API/CLI
nextest RED or proof that whitelist removal alone reaches parity; API was not
executed during discovery and no test runner/comparator was changed.

First-layer near Auto role order in both modes is:
`Skirt → Brim → Outer wall → Brim → Outer wall`.
Far Auto is `Skirt → Brim → Outer wall → Skirt → Brim → Outer wall`.
Thus the near pair proves one skirt group with two independently placed brims,
while the no-brim near control retains two groups. Mesh/placement identical
within the near pair; Auto occupancy is the source of that group change.

### Reproducibility / execution provenance

Evidence contains `generate-inputs.js`, `run.sh`, `summarize.js`, `first-layer.js`,
`inputs/inputs.json`, exact input3MFs, per-case Native logs/result/G-code/exit,
CLI stderr/exit/G-code where successful, `summary.json`, and
`first-layer-summary.json`. Scripts/probes/Native output stay outside Git.

- Official: extracted2.4.2 via
  `${COORD_ROOT}/bin/orca-wrapper-inherit-cpu.sh`, explicit
  `ORCA_APPDIR=${COORD_ROOT}/orca-official-242/squashfs-root`.
- Seeded: `scripts/orca-seeded-parity.sh`, seed0,
  `ORCA_REPLAY_CPU=13`, explicit `/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2/bin/orca-slicer`.
- Every slicing/comparison command is wrapped in `taskset -c 13`, serial modes.
  No cargo/Nix builds, worker threads, stress, CPU48–51 use, or modified sources.
- Read-only existing Ares binary from the original AutoBrim worktree,
  source head `c5ea87946cb8a0a08d483f158b89f2ff10e60228`, binary SHA256
  `ba88fa3b6bd01bf017bf99f727b708a5e157b8c68f52ab0b3d52fd5170758999`.
  Its source branch/HEAD is unchanged. It is **not** the discovery base's CLI.
- This worktree has no initialized upstream submodule; source was read from
  the original worktree's `OrcaSlicer` checkout at
  `8500fcdccaa10b5099ac20d252af3a7c560046f1` without editing it. Cited file
  hashes, Native binary hashes and Ares provenance are recorded.

Replay into a **new** external root, after copying the four scripts there:
`taskset -c 13 node <root>/generate-inputs.js <root>/inputs`, then
`taskset -c 13 bash <root>/run.sh <root>` and
`taskset -c 13 node <root>/first-layer.js <root>`.
The generator reads the retained fixture in the original AutoBrim worktree;
freeze that fixture and verify its hash before replay.

## Proposed post-approval SPEC → RED → GREEN

1. After the dependency lands, rebase this docs-only slice on current main and
   reconfirm these Native observations before writing tests. Approval is needed.
2. RED freezes input-only cases for both distances and the no-brim/combined
   controls. No unused fixtures. Native through `OrcaRunner::from_env().unwrap()`
   with the standard required-oracle environment; no silent optional return.
   Require actual brim/skirt XY extrusion, first-layer group counts and order,
   then unchanged strict whole Native/API/CLI bytes and raw API/CLI equality.
   Both targets must fail at the existing skirt guard; controls must pass.
3. GREEN only production code within the cited consumer/admission boundary;
   preserve tests/docs and all other guard terms. Do not assume a match-only
   change is sufficient; first strict mismatch decides the bounded repair.
4. Native/API/CLI near/far gates in both modes; existing explicit-brim families,
   small-diagonal AutoBrim and negative no-brim controls remain unchanged.
   Final exact hash: fmt, clippy, wasm32 and coordinator-authorized dual-mode
   canonical FULL, with fresh artifacts and assigned CPUs. No FULL during this
   discovery. New RR and merge remain coordinator-only.

## Explicit exclusions / stop conditions

No combined-brim contact union, repeated instances, ByObject expansion, active
support/raft/tower/flush, multi-material cross-group adhesion, object/volume-local
speed overrides, new auto-width rules, brim bed masks, or geometry fallback.
The accepted AutoBrim SPEC′ material/speed limitations remain unchanged, without
claiming general parity or adding an Unsupported guard to enforce this draft.
Native rejection, empty brim motion, unchanged near group count, unexpected
owner order, non-parity controls, or a first raw difference outside this boundary
blocks RED/GREEN approval; do not weaken evidence or alter material_tables/M73
handling. Only this draft document is committed on this discovery branch.
