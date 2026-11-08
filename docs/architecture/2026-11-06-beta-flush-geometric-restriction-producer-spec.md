# Static FlushMode geometric restriction producer

## Decision and ownership

Port `PrintObject::detect_extruder_geometric_unprintables` from
`OrcaSlicer/src/libslic3r/PrintObject.cpp:349–515` at revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. This supplies real geometric
restrictions to the [static FlushMode boundary](2026-11-06-beta-flush-engine-source-boundary-spec.md).
It does not implement enumeration, remembered selection or AMS scoring.

The result is one set of zero-based logical filament IDs per physical head.
`Print.cpp:3593–3604` unions object sets by head. `ToolOrdering.cpp:2907`
obtains that result and `:1537–1561,1588–1589` combines it with physical
restrictions in the grouping context. Geometry is an input to grouping.
It neither changes the filament map directly nor replaces output validation.

The Rust destination is a private producer at
`project_slice/gcode_emit/tool_ordering/recommend.rs::build_recommended`.
It borrows existing prepared geometry and current ordered extrusion entities.
The early all-master recommendation remains a temporary compatibility shell,
not an alternative source for these restrictions. No public API, retained
state, configuration column or motion behavior is added by this producer.

## Included source behavior

| Upstream boundary | Required behavior |
| --- | --- |
| `PrintObject.cpp:351–353` | One physical head produces one empty set. Multi-head results retain physical-head indexing. |
| `:357–371` | Resolve actual per-head exclusion regions, translate them by scaled plate origin, and test actual model instances against the exclusion prisms. A collision prohibits all of that object's printing filaments on that head. |
| `:374–410` | For layers with `print_z` strictly above the head's printable height, use the region's current fill/perimeter entity-family presence to select configured positive filament IDs. |
| `Print.cpp:4995–5006` | Derive each head's unprintable area by subtracting its printable polygon from the bed printable polygon. |
| `PrintObject.cpp:415–424` | Translate those area polygons into object coordinates using the first instance's `shift_without_plate_offset`. Preserve this source distinction from the exclusion-region plate translation. |
| `:441–450` | When fills exist, intersect retained `fill_expolygons` with the head's unprintable area. Bounding boxes only reject work before polygon intersection. |
| `:459–472` | When infill is already unprintable and perimeters exist, prohibit the configured wall filaments without a second wall-area intersection. |
| `:474–488` | Otherwise construct wall area with `diff_ex(raw_slices, fill_expolygons)` and intersect that area with the head's unprintable polygons. |
| `:507–513`; `Print.cpp:3593–3604` | Union restrictions within a head, then across objects. An ID can be forbidden on both heads. The producer does not erase such conflicts. |

Entity-family presence is not a filter to just the filament IDs seen on an
emitted path. If fills exist, the source considers all positive configured
internal-solid, top, bottom and sparse-infill IDs. If perimeters exist, it
considers both positive wall IDs. Convert these one-based configuration IDs
to zero-based set entries exactly once.

Use current ordered-entity membership for those predicates. `fill_entities.rs`
moves perimeter and thin-fill entities out of retained surface records.
An empty moved-from record therefore does not mean that the region has no
printed perimeters. Retained fill polygons still describe the source area.

Build fill and wall polygon temporaries lazily per layer-region and reuse
them across heads. Read raw slices from the retained object/region geometry.
Do not reconstruct filled area from toolpath strokes, replace polygon
intersection with bounding-box overlap, or synthesize empty constraints.
Reuse the existing bed-exclusion parser and triangle/prism intersection.
They remain read-only and are not duplicated by this producer.

## Retained geometry and access boundary

`prepare_infill/surface_type_detection/types.rs::PreparedSurfaceTypeRecord`
owns `fill_expolygons`. The existing horizontal-shell envelope retains those
records and its boxed classic predecessor. Ordered extrusion layers provide
current entity membership, not a replacement copy of the fill polygons.

The envelope access change has exactly four sites in two existing files.
The producer consumes this borrowed data directly in `recommend.rs`.

| File and site | Change |
| --- | --- |
| `tool_ordering/recommend.rs`, traversal-type import | Import `PreparedPostHorizontalShellPropagation` instead of `PreparedPostClassicTraversal`. |
| `recommend.rs`, `build_recommended` parameter | Accept `horizontal: &mut PreparedPostHorizontalShellPropagation`. |
| `recommend.rs`, function entry | Bind `let traversal = &mut horizontal.predecessor`. Existing classic-traversal consumers keep that predecessor. |
| `layers/schedule.rs`, `build_recommended` argument only | Descend five predecessor hops instead of six. Other traversal references are unchanged. |

No other access-site change, new shared accessor, widened visibility, cloned
geometry or retained-state field is part of this boundary. Existing geometry
and option modules remain read-only. The producer's private implementation
must fit the existing file-size limit without adding an access layer.

## Observable contract and failure modes

The public seam remains fixed 3MF bytes through validated API and CLI
slicing against runtime Native output. Existing restriction pairs use the
same model, plate and preset payloads. Only the relevant restriction setting
changes. Both successful endpoints are sampled before target map assertions.

| Existing witness | Discriminated behavior |
| --- | --- |
| Second-head height pair | Native `121→111` when only head 2 becomes too short. The first head remains printable. |
| Second-head area pair | Native `121→111` when only head 2's printable area becomes disjoint from the models. |
| First-head exclusion prism | Native `212` requires the actual exclusion restriction. Ares currently publishes `111`. |
| Three original first-head height/area cases | Native slicing succeeds but Ares returns printable-area validation errors. These are validation outcomes, not successful map witnesses. |

The producer can fail by using moved-from entity presence, confusing logical
filaments with heads, dropping positive configured role IDs, applying the
plate translation to the wrong geometry, losing polygon holes, replacing
wall subtraction with raw slices, accepting a box-only collision, or failing
to union restrictions across objects. Those are separate from solver scoring.

A map mismatch proves missing end-to-end behavior. It does not isolate an
incorrect producer from an absent solver. Producer-only correctness is not
established by unchanged map output, Native literals, source inspection alone
or a claim that the later solver will consume an unused result. No test-only
public accessor or alternate output protocol is part of this design.

Physical masks, declaration-based volume restrictions, scoring, publication,
rematerialization, dynamic/multi-nozzle grouping and independent printable-area
validator repairs remain outside this producer slice. Existing map, import,
retraction and strict whole-output assertions retain their meanings.
