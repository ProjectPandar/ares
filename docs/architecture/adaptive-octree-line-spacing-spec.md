# Adaptive octree line-spacing source semantics SPEC

Status: proposed, docs only. Base: `origin/main=41f2c90f765d42c6620583121296bb61a2cb839c` plus carried collinear STOP docs `87e0fe2b9` and guard STOP note. No Native run, RED, or production edit is authorized by this commit.

## Upstream boundary

`Fill/FillAdaptive.cpp:296-321` establishes the per-region tristate admission.
`PrintObject.cpp:2735` prepares the Support tree on the same object traversal.
`FillAdaptive.cpp:314-350` averages density and extrusion width over every
region whose tristate is `Yes` for the requested pattern. It reads
`print_object.printing_region(0).config().fill_multiline.value` at `:352` and
multiplies both Adaptive and Support spacing at `:353-354`.

`printing_region(0)` is upstream semantics, not an Ares `regions[0]` shortcut.
The source region list is created/merged by `PrintApply.cpp:1000-1012`
(`get_create_region`). The Ares port's corresponding index is assigned at
`print_apply/generate_regions_state.rs:268`, called by
`generate_model_part_region_state.rs:91` and
`generate_modifier_changed_config_state.rs:108`; `group_fills/params.rs:83`
forwards `input.current.region_index` into `SurfaceFillParams`. Fixtures must
use genuinely different configs so `get_create_region` does not merge them.

Bridge candidates are collected by `PrintObject.cpp:2468-2572`; the
`dont_filter_internal_bridges` flag at `:2532` and `:2550` changes filtering,
not candidate collection. The fixture below is concrete and bridge-free:
20×20×10 mm constant prism, two genuinely different Adaptive regions,
`top_shell_layers=0`, `bottom_shell_layers=3`, no support, no modifier solid
volumes, and sparse AdaptiveCubic below the absent top shells. Before Native
runs, inspect every layer's `fill_surfaces` and assert
`filter_by_type(stInternalSolid)` is empty while the sparse surface remains;
this directly checks `PrintObject.cpp:2543-2546`. Thus no internal solid infill
exists over unsupported sparse, and the original-surface triangle path at
`:2728-2735` cannot be reached for this fixture. This is the required
bridge-free verification; absence of an `Internal Bridge` G-code role is not
used as proof.

## Ares destination and replacement boundary

Ares currently keys its Adaptive octree cache per local spacing in
`project_slice/fill_entities/adaptive.rs:101-114`, derives each region's local
spacing in `adaptive/line_spacing.rs:6-28`, and receives region-local values
from `group_fills/params/projection.rs:100-109`. Support trees are independently
prepared in `adaptive/preparation.rs:41-46,71-80`; Support is a second RED
candidate, never a control. W6's coordinate transforms and
`build_object_octree` remain outside this slice.

The planned fix replaces per-region spacing keys with the source-derived
per-object averages and shared region-0 multiline multiplier while preserving
kind separation and cache ownership. No bridge-triangle behavior changes.

## Fixtures, discovery, and RED plan

After SPEC approval, freeze new authored fixtures and a SHA manifest before
any test commit. Each fixture changes exactly one variable:

1. **Adaptive density RED:** the concrete bridge-free prism above, two regions
   both AdaptiveCubic and both contributing surfaces; only one contributing
   region's density differs, with widths and multiline equal.
2. **Adaptive multiline RED:** the same two-region, both-Adaptive geometry;
   only region 0's `fill_multiline` differs from the contributing region's
   value. Density and width are equal. Region 0 is explicitly Adaptive in the
   fixture, even if its surface contribution is absent on a given layer.
3. **Adaptive single-region control:** one Adaptive region with ordinary
   positive settings; strict equality is required.
4. **Support density RED:** a separate two-region SupportCubic fixture where
   only one contributing region's density differs; it is a RED candidate.
5. **Support multiline RED:** a separate two-region SupportCubic fixture where
   only region 0 multiline differs from the contributing region; it is another
   RED candidate, never a control.

For every fixture, perform the per-layer `stInternalSolid` empty check above
before Native. Native official and seeded runs use CPU12 and fresh cache and
artifact roots. Record exit, stderr/result JSON, output SHA, region settings,
and first spacing/tree divergence. Native refusal is classified, not RED.

If Native accepts and Ares differs, public RED generates the authored project
at test time, invokes Native, `slice_project_plate_validated`, and real CLI,
then requires strict whole-output equality and API==CLI raw equality. The
single-region Adaptive control must pass equality. Support cases remain RED
candidates or are explicitly deferred after discovery. No Native output is
committed, and no normalization or retrying is allowed.

## Included and deferred

Included: per-object Adaptive/Support density and extrusion-width averaging
(`FillAdaptive.cpp:314-350`), shared upstream region-0 multiline semantics
(`:352-354`), Ares spacing/cache/preparation destinations, and the separated
single-variable fixture families. Deferred: bridge triangles
(`PrintObject.cpp:1006-1007`), W6 octree coordinate transforms, hook/chain
behavior, multiline geometry expansion, object-region spacing beyond this
source-derived tree spacing, Honeycomb/TPMS-D anchoring, and public contracts.
If no valid Native-admitted spacing divergence is found, stop with NO-RED.

## Amendment required before RED

The bridge-free check excludes the bottom-shell layers themselves. With
`bottom_shell_layers=3`, `discover_horizontal_shells` may produce
`stInternalSolid` on layers 1 and 2 (and the bottom boundary layer); those
solids are expected and are harmless under `PrintObject.cpp:2543-2546`. The
fixture also sets `bottom_shell_thickness` so it does not extend solidification
above those bottom-shell layers. The check therefore starts on the first layer
above the bottom-shell range and verifies the Native G-code there.

Native `fill_surfaces` is not directly observable. The runtime test instead
checks every Native layer above the bottom shells for both absence of
`;TYPE:Internal solid infill` and absence of bridge feature markers. This is
required because `stInternalSolid` always emits as solid infill; the check does
not infer candidate absence from a missing `Internal Bridge` role alone.

The two regions are constructed by one model-part volume followed by one
modifier volume in the authored 3MF object order. `PrintApply.cpp:1000-1012`
`get_create_region` visits the part first, so its config becomes region 0;
the modifier config becomes region 1. The modifier footprint is a 10 mm ×
10 mm rectangle (100 mm²), well above the default `minimum_sparse_infill_area`
of 15 mm², so it is not solidified or discarded as a tiny sparse region. The
part and modifier configs differ only in the fixture variable under test
(density or multiline), and their distinct configs prevent `get_create_region`
from merging them.
