# Support ironing with raft offset: Native divergence proposal

Docs only; no production change or RED. Against
`8f6ce1660ccfaf95a6888b60c258047183d19ff0`. The existing ordinary
support-ironing pattern dispatch and the earlier Grid NO-RED probe are merged;
this is a distinct raft/ironing interaction.

## Native-first witness

Starting from the accepted support-overhang fixture
`tests/parity/support-ironing/overhang-organic.3mf`, the probe changed only
project settings to enable two raft layers and ordinary support:

- `raft_layers=2`
- `support_type=normal(manual)`
- `support_style=grid`
- support ironing remains enabled with the existing rectilinear pattern

The resulting frozen input SHA-256 is
`e361fd095b95efe469220c3927d209538abdfaee1d206220c7d836c86ef78934`.
Native was run first with fresh cache/output roots in both modes:

| mode | result | Native G-code SHA |
|---|---|---|
| official | `return_code=0`, 24 triangles | `de2fd0a41b47189a726380440ee2fd0bf5d28c508c1c6140b5e55eb1d5c23007` |
| seeded | `return_code=0`, 24 triangles | `de2fd0a41b47189a726380440ee2fd0bf5d28c508c1c6140b5e55eb1d5c23007` |

Ares on the same input produced G-code SHA
`16cbbbcdc6fd95e954b73ffdd73b270da319da960328f0feba7df8c59ba9edec`.
The first raw difference is an `M73` placement. Removing all `M73` lines to
isolate motion, the first difference is in the support ironing section:

```
Native: G1 X104.65 Y104.784 Z10.85
Ares:   G1 X104.65 Y104.511 Z10.85
```

The surrounding role is `;TYPE:Ironing`, immediately after the raft/support
layer travel and before the first ironing extrusion. The Native output has
12,513 lines including M73; Ares has 12,512. This is a concrete geometric
start-point divergence in an admitted output, not an UnsupportedProjectFeature
or an upstream-undefined pattern.

No guard was lifted: the input reaches the existing ordinary support-ironing
path. The result is therefore a runtime port gap, and the local probe was
restored byte-for-byte after the comparison.

## Source boundary and proposed slice

The upstream owner is the support-ironing branch in
`OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1877-1903`:
`SupportGeneratorLayer::generate_ironings` constructs the ironing filler,
passes the interface angle/spacing/flow, clips against the immediate upper
support layer, and emits `erIroning` paths. Its raft-layer context is selected
by `SupportCommon.cpp:1440-1523`, where raft/contact layers and their shifted
Z coordinates are planned. The shared fill factory is
`Fill/FillBase.cpp:40-77`; ordinary surface execution is in `Fill.cpp`.

The Ares destination is
`crates/ares-core/src/project_slice/support_material/ironing.rs::IroningSurface::append_paths`
(lines 17-117), called from
`support_material/toolpaths.rs::generate_grid_support_toolpaths` (the cache
modulation/upper-layer handoff around lines 70-111). The adjacent parameter
source is `support_material/toolpath_parameters.rs`, which supplies
`raft_interface_flow` and support-interface height/spacing.

The proposal is to localize and port only the source-equivalent raft-aware
ironing upper-layer reference/offset and start-point construction. It must
preserve ordinary pattern dispatch, flow, role, and clipping semantics. A
future RED will generate Native at test time and assert strict full output,
with a no-raft support-ironing control that remains byte-identical.

Included: the two-raft-layer interaction, support-interface upper trim,
`print_z`/height handoff, and the first ironing path start point. Deferred:
non-rectilinear ironing patterns already covered by the shared ordinary
surface dispatch, Lightning/AdaptiveCubic/SupportCubic Native-UB paths, tree
support ironing, and unrelated M73/time-estimate accounting.

## Localization addendum and RED

A scratch GDB run on the exact frozen input stopped at
`support_material/toolpath_cache/contacts::IroningSurface::append_paths`.
The divergent second call had `print_z=10.449999999999996`,
`height=0.35344827586206884`, `upper=None`, and `angle=1.57079637`; these
values rule out a Z/height mismatch and an upper-layer subtraction. The Ares
call chain is `support_material/toolpaths.rs:98` to
`support_material/ironing.rs:30-117` and then
`fill_entities/surface.rs:122-131`.

The upstream variable is the ironing `Fill` object's `bbox_object`, assigned at
`SupportCommon.cpp:1825-1827` (`f->set_bounding_box(bbox_object)` before
`f->layer_id`, `f->z`, and `f->angle`). The same function sets the source angle
and spacing at `:1827-1833`, unions the saved polygons at `:1833`, and performs
upper-layer clipping only at `:1835-1840`. Ares supplies no corresponding
bbox: `support_material/ironing.rs` constructs `SurfaceCall.reference` as
`Point(0, 0)` and `surface.rs:122-131` passes `LayerGeometry { bounds: None }`
to the Grid filler. Thus the first differing Y is a Grid start-phase caused by
missing `bbox_object`, not by support-layer Z/height or clipping.

The RED commit is `1ff3656f5bbb82849ac8826eb8917037187f252f` and adds the
frozen target fixture plus a no-raft control. Both official and seeded
runs used fresh cache/artifact roots. Each mode
reports the control PASS and the target strict comparison FAIL at the same
first motion/M73 boundary (Native `G1 X115.15 Y114.85 E.03153` versus Ares
`M73 P12 R23` at line 631; the earlier travel difference is
`G1 X104.65 Y104.784` versus `G1 X104.65 Y104.511`). The strict comparator
includes M73, so the RED does not defer it. The no-raft control is unchanged.

The GREEN must pass the whole output, including M73. If correcting the
upstream-equivalent bbox/start phase does not remove the M73 placement
difference, that downstream time-estimate path becomes a separate localized failure
within this same RED and cannot be ignored.

## Localization addendum (gdb, exact RED input)

The release Native binary was run under `/nix/store/fq5hd83bkipznh5n0m3zxcs4gk6hcbjc-gdb-17.2/bin/gdb`
with the official AppDir runtime library path. A breakpoint on
`Slic3r::FillRectilinear::fill_surface` (the return-by-value ABI has the
output vector in `rdi`, Fill in `rsi`, Surface in `rdx`, and FillParams in
`rcx`) captured the support-ironing calls. The first raft call is layer 1;
the later calls are layer 30, the layer containing the RED first difference.
The layer-30 call has the following exact Native inputs:

```
density=1, multiline=1, anchor_length=1000, anchor_length_max=1000,
resolution=0.0125, dont_adjust=true, spacing=0.1, overlap=0,
angle=1.5707963, link_max_length=300000 (scaled), bbox=(-1000000,-1000000)..(1000000,1000000)
```

The same capture shows `SupportCommon.cpp:1456`'s scaled ±1 mm box, so its
center is `(0,0)` and there is no instance shift. `SupportCommon.cpp:1870-1877`
sets the Fill fields above; `:1879-1880` creates the input surface by applying
`union_safety_offset_ex` to `layer_cache.polys_to_iron`.

Ares was traced at `IroningSurface::append_paths`: the first call is
`layer_id=1`, `print_z=0.5`, `height=0.3`, and the second is
`layer_id=30`, `print_z=10.449999999999996`, `height=0.35344827586206884`;
the angle is `1.57079637`. The layer-1 surface has one polygon and the
layer-30 surface has two polygons. `support_material/ironing.rs` sets the
same density/spacing/angle/dont-adjust/link values as the Native call.

Therefore the first differing input field is the layer-30 ironing surface
expolygon collection (polygon coordinates/count/order), before the Fill
parameters are consumed. This is upstream-owned at
`SupportCommon.cpp:1659` (`layer_cache.polys_to_iron =
 top_contact_layer.polygons_to_extrude()`) and `:1879` (union before Fill);
the Ares destination is the `IroningSurface.polygons` construction in
`support_material/toolpath_cache/contacts` consumed by
`support_material/ironing.rs::IroningSurface::append_paths`. The bbox and
FillParams hypothesis is falsified. GREEN must first port the layer-30
contact polygon source/ordering; M73 remains in-scope after that geometry
matches.

### Native polygon-count confirmation

A second gdb breakpoint on `Slic3r::union_ex(std::vector<Polygon> const&)`
(the `union_safety_offset_ex` implementation reached from
`SupportCommon.cpp:1879`) captured the layer-30 call in the support-toolpath
worker (`generate_support_toolpaths` `$_2`). Its input vector has **N=1**
polygon, with four points (scaled coordinates):

```
( 7150587,  7150587)
(-7150572,  7150587)
(-7150572, -7150572)
( 7150587, -7150572)
```

Thus Native's layer-30 `polys_to_iron` count is one, point count four, and
bbox is min `(-7150572,-7150572)` / max `(7150587,7150587)`. Ares' corresponding
layer-30 `IroningSurface.polygons` count is two. This pins the first differing
input to the pre-union collection, rather than FillParams or bbox center.
