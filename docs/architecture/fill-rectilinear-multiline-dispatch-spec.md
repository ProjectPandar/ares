# SPEC: Rectilinear sparse multiline dispatch

Status: proposed.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Selection and ownership

Port the remaining Rectilinear dispatch finding R2 from
[the shared multiline union slice](fillbase-multiline-clipper2-union.md).
Historical Native/Ares first whole-file difference was line18
`M73 P0 R38` / `M73 P0 R37`. This is historical evidence only,
not RED on the new base. Grid mode0 and Stars mode2 prerequisites are merged.

Read-only main inspection still finds both Rectilinear consumers constructing
`MonotonicFillParams`, which has no multiline field. Per-object skirt,
counterbore non-None and OptionsZCorrector production paths are excluded. The
proposed adjacent bridge-anchor change is confined to the Rectilinear dispatch
sub-branch in sparse_anchoring/arms.rs. No changes
to anchor_projection, candidate expansion, admission, bridge classification or
other pattern arms are included.

## Upstream source boundary

| Source | Behavior to preserve |
| --- | --- |
| `PrintConfig.cpp:2996-3002` | Existing fill_multiline integer1..10; no option schema or inventory change. |
| `Fill/Fill.cpp:926` | Only internal sparse infill receives the regional multiline count; other extrusion roles get1. |
| `Fill/FillBase.hpp:50,52` | full_infill is density>0.9999f; dont_connect is anchor_length_max<0.05f. These are exact source predicates. |
| `Fill/FillRectilinear.cpp:3387-3399` | Rectilinear uses ordinary by-lines for full infill/count1 and the named zag exceptions; otherwise calls by-multilines with exactly one sweep `{0.f,0.f}`. |
| `Fill/FillRectilinear.cpp:2996-3017` | Expanded surface, scaled integer line width/spacing, source float direction/reference, single-sweep scanline generation. |
| `Fill/FillRectilinear.cpp:3020-3045` | multiline_fill then half-spacing contraction, source fallback on empty contraction, ordered clipping and connect-or-chain. |
| `Fill/FillBase.cpp:275-319` | PrintObject bbox center, bridge override, effective layer/thickness alternation unless fixed, then f32 pi/2 accumulation. Rectilinear must retain its alternating directions, unlike Grid's fixed sweep frame. |
| `Fill/Fill.cpp:1224-1237,1281-1303,1354` | Normal emission installs the object bbox and carries density/multiline/pattern/flow to the filler. |
| `Fill/Fill.cpp:1394-1449,1477-1498` | Sparse-anchor generation invokes the same Rectilinear filler with overlap0, actual object/layer identity and multiline. |
| `Fill/FillBase.cpp:2712-2784` | Existing multiline bundle expansion and Clipper2 offset/Positive union; use the already merged private numeric policy, not a new clipping kernel. |
| `Fill/FillRectilinear.cpp:3401-3419`; `FillRectilinear.hpp:38-46` | Monotonic subclasses remain by-lines. AlignedRectilinear inherits a different angle policy and is explicitly deferred here. |

## Rust destination

`project_slice/group_fills/params/projection.rs:91-97` already projects the
source sparse-only multiline count; keep it unchanged.
`project_slice/fill_entities/monotonic.rs::append` currently sends Rectilinear
to `fill_rectilinear_surface` regardless of count. It is a temporary by-lines
compatibility shell for the upstream count1/full branch, not a multiline port.
`prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs:210-279` has the same
missing dispatch in its shared linear-pattern arm.

Proposed bounded destinations after fresh RED:

- New private `fill/rectilinear_multiline.rs`: source dispatch eligibility and
  the one-sweep adapter over existing `fill::multiline::fill_surface`. Preserve
  the precise `_infill_direction` f32 accumulation, effective layer id,
  representative thickness, fixed-angle and object-center reference. No new
  public API, general pattern framework or additional bbox producer.
- `fill_entities/monotonic.rs`: a Rectilinear-only count>1/non-full branch,
  delegating multiline collection materialization to a private helper if needed.
  Preserve role, flow, spacing, collection sorting, residual-gap lifecycle and
  whole-file emission. Count1/full and all other existing expressions stay on
  the existing path. Do not mark Oriented/Path heuristically to force equality.
- `sparse_anchoring/arms.rs`: the same Rectilinear-only eligible sub-branch
  with source overlap0 and its actual effective layer/thickness. The existing
  shared by-lines arm remains for the other patterns and controls.
- Module registration plus a dedicated public runtime Native E2E test module
  under `ares-cli/tests/orca_parity`. Reuse existing fixture adapter/slice_case
  and full comparator; do not alter shared runner/cache contracts.

The current multiline generator is reused unchanged initially. If new RED
reveals an independent numeric, clipping, bbox, shared generator or flow
defect, stop and report its exact stage/source boundary before widening scope.
Every new or changed file stays at most400 LOC; purely mechanical extraction,
if required, must be a separate commit with exact-text reconstruction.

## Included and deferred

Included: standard Rectilinear internal sparse infill count2..10; both emission
and anchor consumers; default direction alternating with effective layer id;
existing connected and dont_connect behavior where witnessed. One object,
instance, volume, region and filament; constant-height classic walls, no tower,
raft, support, modifier, ironing, arc fitting, fuzzy skin, scarf or SAFC.

Deferred: AlignedRectilinear inheritance, Line/Zag variants, Triangles mode1,
new high-density or degenerate clipping defects, rotated multi-instance bbox
changes, variable-height/range configuration and support/modifier interactions.
No rejection is added for deferred inputs. Do not remove any existing guard,
change an existing negative expectation, fix counterbore or alter skirt/web paths.

## Fresh Native-backed RED plan

Use the tracked authored prism/holed-prism STL inputs from
`tests/parity/multiline-union`, exported at runtime through OrcaRunner with
existing Ender-3 presets and verified requested settings. Native accepts and
produces the complete reference independently in official and seeded modes.
Both modes get fresh roots/caches. No Native output is committed.

Freeze twelve controls before production:

| Controls | Purpose |
| --- | --- |
| R1/R2/R3/R10: prism, Rectilinear count1/2/3/10, density30%, direction45 | Historical R2 replay plus even/odd/max legal bundle witnesses and count1 control. |
| R2A0: same prism, count2, direction0 | Alternate frame on successive sparse layers, rather than a fixed Grid frame. |
| RH1/RH2: holed prism, count1/2 | Ordered clipping around a hole; count1 control. |
| RF100: prism, Rectilinear density100%, count2 | Full-fill source branch and sparse-only projection remain unchanged; do not assert a multiline effect in this control. |
| M2/Z2: prism, Monotonic/ZigZag count2 | Unchanged by-lines subclass/zag exception controls. |
| G2/S2: prism, Grid/Stars count2 | Already merged multiline consumers remain exact; not new RED candidates. |

Each case requires verified exported option values, successful Native slicing,
nonempty real sparse/solid extrusion as appropriate, and unchanged full ordered
generator-only byte comparison with line counts for public validated API and
CLI. Record whole-file first difference and first differing sparse layer/path;
a leading M73 difference alone cannot attribute the cause to this dispatch.
Observe actual bundle geometry/direction and surface role. Do not compare only
a bbox, polygon summary, footer or selected lines in place of full bytes.

RED must demonstrate a real Native-admitted main mismatch in an eligible
Rectilinear case. If every eligible case is equal, report NO RED FOUND and stop.
If any noneligible control already differs, preserve its evidence and classify
it before modifying controls or their comparison. Commit RED separately before
GREEN.

## Acceptance

Eligible RED cases must become whole-file PASS in both modes. Preserve every
count1/full/other-pattern control's Ares raw bytes across RED and GREEN;
report any unrelated raw changes before proceeding. Assertions and comparator
remain unchanged. Run full option_coverage in both modes, strict clippy
`--workspace --all-targets --all-features -- -D warnings`, fmt and wasm32.
No ignore, suppression, probe, regions[0], new Unsupported gate,
oracle fallback or Native artifact submission is permitted.
