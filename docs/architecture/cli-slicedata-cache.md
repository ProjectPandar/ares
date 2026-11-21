# CLI slicedata cache

Port the OrcaSlicer v2.5.0 slicedata cache at upstream `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. The source boundary is `src/libslic3r/Print.cpp::Print::export_cached_data` and `load_cached_data`, with the CLI dispatch in `src/OrcaSlicer.cpp`. This is a cache of an already-sliced `PrintObject`; it is not a project archive and it must not become a second slicing pipeline.

## Native contract

`Print.hpp:1088-1089` declares the two operations and `PrintBase.hpp:437-438` provides the virtual defaults. `PrintConfig.cpp:12740-12750` registers `export_slicedata` and `load_slicedata` as directory-valued options. The cache is JSON, not cereal or a binary archive. The `to_json`/`from_json` helpers are local to `Print.cpp:5987-6615`; no cereal helper participates in this format.

### Directory and files

The CLI chooses a one-based plate directory:

```text
<slicedata-root>/1/obj_<identify_id>.json
<slicedata-root>/2/obj_<identify_id>.json
```

`OrcaSlicer.cpp:7093-7113` loads `<root>/<plate>` and `:7245-7261` exports to the same path. `export_cached_data` first removes and recreates the leaf directory (`Print.cpp:6640-6710`), then writes one file for every non-shared print object. The parent root is not recursively created by this function, so the CLI adapter must create or validate the root before invoking it. A shared object is skipped (`:6715-6725`). The file identity is `loaded_id` when positive, otherwise the model instance ID (`:6715-6725`, `:6919-6946`).

Each file is terminated by a newline. `with_space == false` uses `json.dump(0)`; `with_space == true` uses one tab of indentation (`Print.cpp:6882-6893`). The CLI selects `with_space` from its logging level (`OrcaSlicer.cpp:7251-7253`). File order and object-key order are not a compatibility contract; the object ID in the filename is.

### JSON envelope

The root object has these keys, emitted by `Print.cpp:6715-6855`:

| Key | Shape and meaning |
| --- | --- |
| `name` | Model object name. |
| `identify_id` | File identity selected from the model instance. |
| `layers` | Ordered ordinary `Layer` records. |
| `support_layers` | Ordered `SupportLayer` records. |
| `first_layer_groups` | Ordered grouped-volume records. |

`tree_support_layers` exists as a key constant (`Print.cpp:5910-5913`) but is not emitted by v2.5.0. A compatible reader must not invent it or treat its absence as a serialized empty tree-support collection.

An ordinary layer is created by `convert_layer_to_json` (`Print.cpp:6645-6688`) and contains:

- `print_z`, `height`, `slice_z`, and `layer_id`;
- ordered `sliced_polygons`, each an `ExPolygon`;
- ordered `sliced_bboxes`, each `[min_x, min_y, max_x, max_y]`;
- ordered `overhang_polygons` and `overhang_bbox`;
- ordered `layer_regions`.

The geometry helpers (`Print.cpp:5987-6077`) define the wire representation. Points are flat numeric `x,y` pairs; an `ExPolygon` is `{contour: [...], holes: [[...], ...]}`; a `Surface` contains `expolygon`, `surface_type`, `thickness`, `thickness_layers`, `bridge_angle`, and `extra_perimeters`. The 2D `Polyline` helper used for unsupported bridge edges contains `points` and ordered `arc_fitting` records. An extrusion path's `polyline` is a `Polyline3` array of ordered `[x,y,z]` points. An arc record contains its arc flags, angles, endpoints, direction, radius, center, and fitting indices.

A `LayerRegion` is serialized by `to_json(LayerRegion&)` (`Print.cpp:6142-6240`) with:

- `config_hash` for matching the project print region;
- `slices` and `raw_slices`;
- `thin_fills`;
- `fill_expolygons`, `fill_surfaces`, and `fill_no_overlap_expolygons`;
- `unsupported_bridge_edges`;
- `perimeters` and `fills`.

The extrusion representation is recursive (`Print.cpp:6079-6140`). An entity has `entity_type` of `path`, `multipath`, `loop`, or `collection`. A collection carries `no_sort` and `entities`; the other entity forms carry `paths`. Each path carries `polyline`, `mm3_per_mm`, `width`, `height`, `role`, and `no_extrusion`. A loop additionally carries `loop_role`. Arrays are ordered and must not be sorted.

A support layer reuses the ordinary layer fields and adds `interface_id`, `support_type`, `support_islands`, and `support_fills` (`Print.cpp:6760-6820`). A first-layer group contains `group_id`, `volume_ids`, and `slices` (`:6241-6258`, `:6840-6870`). During export volume IDs are converted to indexes within the shared model object's volume vector; during load those indexes are mapped back to model volume IDs (`:6840-6870`, `:7090-7110`).

### Load semantics

`load_cached_data` clears ordinary and support layers before locating matching object files (`Print.cpp:6919-6950`). It parses files in parallel (`:6950-6985`), creates layers and regions, then extracts layer payloads, support payloads, and first-layer groups (`:6995-7129`). A region's `config_hash` must match a region already present in the target project; otherwise loading fails with the cache-not-usable result (`:7015-7032`). Missing files are skipped, which is how shared objects remain outside the cache.

The load path is followed by `print->process(nullptr, true)` (`OrcaSlicer.cpp:7093-7113`), not by normal slicing. A failed load falls back to normal processing in Native. Ares must expose the same choice to the CLI adapter and must never silently use partially loaded state.

## Ares boundary and API

The existing byte boundary is `ares-core::slice_project*` in `crates/ares-core/src/project_slice.rs:102-190`, which accepts `impl AsRef<[u8]>` and returns G-code bytes, and `load_project` in `crates/ares-core/src/project/load.rs:72-90`, which accepts bytes. `ares-core` has no filesystem or terminal ownership. Its current print model is `PrintObject/PrintLayer/LayerRegion` in `crates/ares-core/src/print.rs:26-140`, but it does not yet represent all Native cache fields.

Add the cache as a byte-only core seam, using a Native-compatible object value rather than a path:

```rust
pub struct SliceDataObject { /* name, identify_id, layers, support, groups */ }

pub fn export_slicedata(
    object: &SliceDataObject,
    format: SliceDataFormat,
) -> Result<Vec<u8>, SliceDataError>;

pub fn load_slicedata(
    cache: impl AsRef<[u8]>,
    target: &mut Print,
) -> Result<(), SliceDataError>;
```

The exact Rust names may follow the existing crate naming, but the contract is fixed: export returns bytes, load consumes bytes and populates an already prepared project/print target, and neither function accepts a filesystem path. `SliceDataObject` must carry the Native envelope metadata and all state needed by the selected cache slice. `SliceDataFormat` selects compact or one-tab-indented JSON; it does not add a version key to the Native envelope.

The core owns parsing, validation, numeric conversion, region-hash matching, reconstruction of layer links, and the in-memory print state. The CLI adapter owns `--export-slicedata`/`--load-slicedata` argument admission, root and one-based plate directories, object filenames, file reads/writes, cleanup on failure, and mapping the selected plate's project bytes into the core call. The CLI must preserve the Native mutual exclusion checks at `OrcaSlicer.cpp:6306-6325` and `:6433-6440`: load and export cannot be combined, and load cannot be combined with repetitions.

The byte API must return a typed error for malformed JSON, missing required fields, non-finite numbers, an unknown entity type, a region-hash mismatch, an invalid volume index, or an unsupported non-empty deferred section. It must not fall back to normal slicing inside `ares-core`; fallback policy belongs to the CLI orchestration layer.

## Comparison and acceptance

### Cache comparison

The normative comparison is parsed JSON, not raw bytes:

1. Compare the directory file set and `obj_<identify_id>.json` names exactly.
2. Parse every JSON file and compare recursive object key sets without considering object key order.
3. Preserve every array's order; do not sort layers, regions, entities, paths, points, volume IDs, or groups.
4. Normalize only JSON lexical differences: whitespace, trailing newline, and numeric spelling that parses to the same finite numeric value. Integer values remain exact. Do not apply a tolerance and do not round coordinates or extrusion widths.
5. Canonicalize path separators and compare the plate number and object ID separately from the host path.
6. Reject non-finite values rather than copying undefined or platform-dependent state.

Raw bytes are comparable only when both sides use the same `with_space` mode and the same canonical key insertion order. A byte mismatch caused only by indentation, a final newline, or object-key order is not a cache semantic failure. Ares should still provide a canonical serializer so repeated Ares export is deterministic.

The first Native receipt used a synthetic single-plate cube project. It produced exactly one cache file, `1/obj_14.json`, of 52,424 bytes (6,382 lines); no other cache file was produced. Parsed structure was: `name = cube10.stl`, `identify_id = 14`, 10 ordinary layers, zero support layers, and one first-layer group. Layer 0 had one region, one sliced polygon, one sliced bounding box, and zero overhang polygons. Its region had one slice and one raw slice; its perimeter entity was a collection containing one loop. The cache output is retained outside the repository and must not be committed.

### G-code proof after load

Use a fixed synthetic project and the same profile for all four runs:

1. Native normal slice → `N0`.
2. Native export, then Native load of the cache → `N1`.
3. Ares normal slice → `A0`.
4. Ares load of the Native cache, then process the loaded state → `A1`.

Also run Ares export followed by Ares self-load to prove the serializer/parser round trip. Compare `N1` with `A1` after the repository's approved G-code normalization: normalize line endings and trailing whitespace, preserve line order and all motion values, and remove only explicitly documented generator metadata differences. Never sort G-code or apply a numeric tolerance. The acceptance assertion is that the first meaningful line and byte offset of every remaining difference are reported, and that the loaded outputs are identical for the supported slice.

`N0` versus `N1` proves Native's cache load does not change output. `N1` versus `A1` proves Ares can consume the Native cache and produce the same post-load G-code. `A0` is a separate baseline check; it must not be used to hide a cache-load discrepancy.

## Slices

### Slice 1: one ordinary object, one or more ordinary layers

Implement the smallest complete ordinary-layer cache that can reach G-code generation:

- one object file with `name`, `identify_id`, `layers`, `support_layers: []`, and `first_layer_groups: []`;
- layer metadata, sliced polygons, sliced bounding boxes, overhang polygons/bbox, and ordered regions;
- region hash, slices, raw slices, thin fills, fill polygons/surfaces, unsupported bridge edges, perimeters, and fills;
- recursive path, multipath, loop, and collection entities, including polyline arc-fitting data;
- compact and one-tab-indented JSON output and strict parsed-JSON comparison;
- one selected plate through the existing CLI plate/output dispatch.

For Slice 1, a non-empty support layer or first-layer group is an explicit deferred-input error rather than silently dropped data. Multiple ordinary layers and multiple regions are included; multiple objects, shared-object skipping, and volume-ID remapping should be tested once the single-object path is proven.

### Deferred slices

- non-empty `support_layers`, including support-island and support-fill G-code;
- tree support state (the v2.5.0 exporter does not write `tree_support_layers`);
- non-empty `first_layer_groups` and the model-volume index remapping;
- multi-object and shared-object cache orchestration;
- cache export/load for a plate set rather than one selected plate;
- changing cache schema/version or adding Ares-only envelope keys;
- loading a cache without the matching project region configuration.

The CLI integration should depend on the B60 plate/output dispatch work after it lands. Do not duplicate plate selection, output naming, or saved-output routing in this slice; migrate the cache flags onto that dispatch seam.

## State that must not be ported

The cache contains data needed to resume processing, not every field on the Native classes. Do not serialize or reconstruct these transient fields:

- `Layer::upper_layer` and `lower_layer` links, the owning `PrintObject*`, and other owner pointers (`Layer.hpp:137-160`); rebuild links in layer order as Native does (`Print.cpp:7005-7015`).
- `Layer::slicing_errors`, `curled_lines`, `sharp_tails`, `cantilevers`, and `sharp_tails_height`, which are diagnostics or runtime avoidance data (`Layer.hpp:146-160`); `slicing_errors` is explicitly commented out in `convert_layer_to_json` (`Print.cpp:6650-6653`).
- `LayerRegion::m_layer` and `m_region` pointers (`Layer.hpp:47-132`); retain only the config hash and serialized region payload.
- `SupportLayer::base_areas` and other tree-support-only runtime state (`Layer.hpp:302-335`); do not infer it from ordinary support fields.
- `ExtrusionPath::overhang_degree`, `curve_degree`, `smooth_speed`, and `z_contoured` (`ExtrusionEntity.hpp:174-190`); `to_json(ExtrusionPath&)` writes only the polyline, volumetric rate, width, height, role, and no-extrusion flag (`Print.cpp:6070-6077`).
- Any uninitialized or platform-dependent padding, pointer value, diagnostic cache, or derived geometry not named by the JSON helpers.

Native's support-layer constructor accepts a `slice_z`, but the load constructor call does not supply it (`Print.cpp:7055-7056`); the subsequent extraction does not restore it either (`:6537-6615`). This is an upstream omission, not a compatibility requirement. Ares must preserve support `slice_z` in its own state or reject non-empty support caches until that slice is specified; it must not reproduce an uninitialized/default support height as a feature.

## Out of scope

This document does not add implementation, fixtures, committed Native cache files, cereal serialization, filesystem access to `ares-core`, normal-slice fallback inside the core, or a new project/archive format. The flags remain rejected until this spec is approved and the implementation is authorized.
