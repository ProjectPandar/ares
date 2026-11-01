# IMEX arrange on OrcaSlicer v2.5.0-beta

Every `file:line` is at OrcaSlicer `345efec64`. This ARD covers the IDEX/IQEX arrange of the plater session, in three slices:
- **A2a:** the libslic3r arranger kernel (this section).
- **A2b:** the shared bed-exclude volumes in `ArrangeJob`.
- **A2c:** the plate IMEX zones feeding the plater session and the web arrange.

It extends the "Auto-arrange on OrcaSlicer v2.5.0-beta" decisions in `web-plater-move-rotate-arrange.md`.

## A2a: IMEX arranger kernel

### Oracle: GUI-only, so the upstream unit tests are the oracle
- **Only the GUI calls the arranger.** `ArrangeJob::prepare_imex_zones` snapshots each plate's zones (`src/slic3r/GUI/Jobs/ArrangeJob.cpp:601-626`), and `ArrangeJob::process` builds `ImexArranger imex(m_imex, params, bedpts)` (`:743-744`).
- **The CLI has no IMEX path.** The CLI global arrange (`src/OrcaSlicer.cpp`) never names `ImexArranger`, `ImexArrangeInput` or an IMEX zone. So no Native CLI run can witness this kernel, and A2a has no Native CLI oracle.
- **The oracle is upstream's own unit test of the kernel**, `tests/libslic3r/test_imex_arrange.cpp`.
  - Its 9 `TEST_CASE`s are ported literally: every fixture, input and expected value is copied as written, none is derived.
  - Each case cites its upstream line.
- **`test_imex_zones.cpp` is not needed for A2a.** Its 14 `TEST_CASE`s (`:113-528`) all test `compute_imex_zone_layout` (config → `ImexZoneLayout`), which A2a does not port. None of them touches `ImexArranger`. They come with A2c, where the plate zones are ported.

### Upstream boundary (ported)
`src/libslic3r/IMEXArrange.hpp`, all but `imex_arrange_zones` and `read_config`:
- `ImexArrangeZones` (`:15-20`): `primary_zone` and `collision_zones`, in plate-local mm.
- `ImexArrangeInput` (`:25-40`): `beds`, `adds_plates`, `added_plate`, `brim`.
- `ImexArranger` (`:42-99`): the constructor, `active`, `bed_shape`, `add_keep_outs`, `finish`, and the private `room_in` and `crosses_keep_out`.

`src/libslic3r/IMEXArrange.cpp`:
- The anonymous-namespace helpers (`:25-63`): `kFenceName` / `kStripName`, `same_zone` (`is_approx` of both corners, `Point.hpp:411-415`: each axis `|d| < EPSILON`), `inset` and `boxes_around`.
- The constructor (`:92-125`): the shared room when every bed, and an added plate, has the same zone; the `MAX_NUM_PLATES` beds an arrange may add; the fences and the strips grown by the bed margin plus `brim`.
- `room_in` (`:127-135`), `bed_shape` (`:137`), `add_keep_outs` (`:139-150`) and `crosses_keep_out` (`:152-158`).
- `finish` (`:160-218`): unarrange a part across a keep-out; re-arrange each fenced plate inside its zone without the fences; keep the first layout when the second does not fit; carry the item ids over.
- `MAX_NUM_PLATES` is 36 (`deps_src/libnest2d/include/libnest2d/common.hpp:14`). The Ares nest port already holds it (`arrangement/nest/selection.rs`).

### Ares destination
- **`crates/ares-core/src/arrangement/imex.rs`**, public as `ares_core::arrangement::imex`:
  - `BoundingBoxf` (mm `min`/`max`, the `BoundingBoxf` upstream passes zones in);
  - `ImexArrangeZones`, `ImexArrangeInput`;
  - `ImexArranger::{new(input, params, bed: &[Point]), active, bed_shape, add_keep_outs, finish}`.
  - `bed_shape` returns the outline points, as upstream's `Points`. The caller arranges on `Bed::from_points` of them, as `arrange(items, fixed, Points, params)` dispatches (`Arrange.cpp:1259-1288`).
- **Internals.** It uses the existing `arrangement::arrange`, `ArrangePolygon::transformed_poly` and the crate's Clipper2 intersection for `crosses_keep_out`.
  - Upstream's `scaled(BoundingBoxf)` is `scaled<coord_t>` of each corner (`BoundingBox.hpp`), i.e. `coord_t(v / SCALING_FACTOR)`.
  - `MAX_NUM_PLATES` moves from `nest/selection.rs` to `arrangement` so the nest and the arranger share it.
- **Scaffold.** No existing scaffold is replaced. `plater/arrange_job` keeps rejecting what it rejects today until A2b/A2c.

### Included and deferred
- **Included:** everything `test_imex_arrange.cpp` reaches, which is all of `ImexArranger` and the two structs.
- **Deferred:**
  - **`imex_arrange_zones(const ImexZoneLayout&)`** (`IMEXArrange.cpp:65-73`) and **`ImexArrangeInput::read_config`** (`:75-90`) go to **A2c**. They need `compute_imex_zone_layout` (`IMEXZones.cpp`) and the IMEX config helpers (`IMEXHelpers`); `read_config` also reads `brim_type` / `brim_width` / `brim_object_gap`. No `test_imex_arrange.cpp` case calls them.
  - **`finish`'s cancelled-pass return** (`IMEXArrange.cpp:196-198`, `one_plate.stopcondition`) has no counterpart: the Ares `ArrangeParams` carries no stop condition and an arrange cannot be cancelled. The branch stays out; it is not stubbed.
  - **`ArrangeJob::prepare_imex_zones`** and the call in `process` go to **A2c**. The **bed-exclude-volume** modes (`ArrangeJob.cpp:57, 89`) go to **A2b**.

### RED: `crates/ares-core/tests/arrange/imex.rs`
- **Where.** It is a new module of the `arrange` test binary. It reaches only the public `ares_core::arrangement` API: no mocks and no crate-internal access.
- **Fixtures** (`test_imex_arrange.cpp:21-96`), copied as written:
  - `kBed` (0,0)–(300,200), `kLeftHalf` (0,0)–(150,200), `kRightHalf` (150,0)–(300,200), `kStrip` (120,0)–(150,200);
  - `kCopy` = {left half, no strips}, `kMirror` = {left half, the strip};
  - `test_params`: `bed_shrink_x/y` 1, `allow_rotations` false, the other fields at their defaults;
  - `shrunk_bed`: the scaled box (1,1)–(299,199) as polygon points;
  - `part(w, d)`: the scaled centred box, `inflation` scaled(1), `extrude_ids` [0], `height` 10, `bed_idx` 0;
  - `squares(n, s)`: `itemid` i;
  - `arrange`, as upstream drives the arranger;
  - `extents`: the union box of the transformed polygons, unscaled.
- **`crosses(ap, box)`.** Upstream intersects the part's transformed contour with the box polygon (`:91-94`). The test reaches no public Clipper call, so it tests the strict overlap of the part's box with `box` instead. This is the same predicate here: `allow_rotations` is false, so every part stays an axis-aligned rectangle, the strip is an axis-aligned box, and Clipper's intersection of two such rectangles is non-empty exactly when they overlap with positive area. Only the helper changes; no expectation does. The equivalence holds only while nothing rotates and every part is a rectangle: if `test_params` ever allows rotation, or a part stops being a rectangle, `crosses` must go back to the Clipper intersection of upstream.
- **Cases.** Each Catch `SECTION` runs from its case's fresh setup, as Catch reruns the case for each one.

| Ares test | Upstream case (`test_imex_arrange.cpp`) |
|---|---|
| `imex_arranger_changes_nothing_without_zones` | `:98-110` |
| `imex_plates_sharing_a_zone_arrange_inside_it_clear_of_the_strip` | `:112-135` |
| `imex_parallel_plate_beside_a_primary_plate_is_centred_in_its_zone` | `:137-157` |
| `imex_primary_plate_beside_a_parallel_plate_uses_the_whole_bed` | `:159-170` |
| `imex_part_too_big_for_its_zone_is_left_unarranged` (2 sections) | `:172-189` |
| `imex_strip_keeps_room_for_the_bed_margin_and_the_brim` | `:191-205` |
| `imex_added_plate_takes_the_zones_it_will_have` (4 sections) | `:207-248` |
| `imex_zone_narrower_than_its_margins_takes_no_parts` | `:250-260` |
| `imex_parts_spilling_onto_a_new_primary_plate_use_its_whole_bed` | `:263-287` |

- **Expected values**, verbatim:
  - the bed-shape corners `scaled(1,1)` / `scaled(149,199)`;
  - the strip box `scaled(114,-6)`–`scaled(156,206)`;
  - the fixed-item counts and `bed_idx`, and `UNARRANGED`;
  - the `WithinAbs` centres (75 ± 5, 100 ± 5, 150 ± 5, 150 ± 10).
- **RED commit.** It adds the public types with an `ImexArranger` whose methods panic ("IMEX arrange is not ported"). All 9 cases then compile and fail, including the no-zone case, which a no-op would pass silently.
- **GREEN.** The port above makes all 9 pass. `cargo nextest run -p ares-core -E 'binary(arrange)'` stays green for the existing 25 cases.

## A2b: bed-exclude volumes in the arrange

Every `file:line` is at `345efec64`. `bed_exclude_volumes` are 3D keep-outs: a bed polygon with an optional z range, per mode (Shared, ToolheadOffset or PerExtruder) and per extruder. They sit beside the legacy 2D `bed_exclude_area`, which A3a already ports.

### Two slices, by oracle
- **A2b1: the config resolution and the arrange blockers.**
  - The CLI global arrange calls `PartPlateList::preprocess_exclude_areas(unselected, m_print_config, …)` (`src/OrcaSlicer.cpp:5484, 5516, 5727, 5958, 5991`), so this path has a **Native CLI oracle**.
  - A probe on `combined-brim.3mf`, with the volume `0..250;0x0,110x0,110x220,0x220` (the bed's left half), moves Native's `--arrange 1 --allow-rotations=0` pile from x 81.5–138.5 to x 139.7–196.7, clear of the volume. Ares today ignores the key.
  - The region resolution also has **literal upstream unit tests** (`tests/libslic3r/test_exclusion_volumes.cpp`).
- **A2b2: the plate-side checks** (its own section below; the per-polygon nozzle ids move to A2b3).
  - `check_outside`'s config-region branch (`PartPlate.cpp:4562-4601`) reads `m_plater->config()`, so it runs in the GUI only; the CLI keeps the `m_exclude_bounding_box` fallback (`:4603-4612`).
  - It needs `ModelInstance::intersects_bed_exclude_region` (`Model.cpp:3753-3836`).
  - The per-polygon `bed_exclusion_extruder_ids` (`ArrangeJob.cpp:53-103`) and `bed_exclusion_extruder_for_filament` (`PrintConfig.cpp:14187-14209`) are GUI-only as well.
  - Their oracle is the upstream Model and filament-mapping cases (`test_exclusion_volumes.cpp:486, 542-574`).
  - Until A2b2, the session keeps refusing a prime tower beside an exclusion, now for a volume too, since the tower's GUI outside check would see it.

### A2b1 upstream boundary
- **`PrintConfig.hpp`:** `BedExcludeVolumeMode` (`:642-646`) and `BedExcludeRegion` (`:2545-2558`).
- **`PrintConfig.cpp`, the region resolution** (the `DynamicPrintConfig` overloads the arrange reads):
  - the parsers: `looks_like_bed_exclusion_volume_syntax` (`:13887`), `parse_double_token` (`:13892`), `parse_z_range_token` (`:13904`), `parse_exclude_polygon_points` (`:13940`), `append_legacy_bed_exclude_regions` (`:13973`) and `append_bed_exclusion_volume_regions` (`:13991-14054`);
  - the resolution: `collision_volumes_configured` (`:14056-14069`), `parse_master_bed_excluded_regions` (`:14071-14087`), `resolve_bed_excluded_regions` (`:14111-14148`), `bed_exclusion_extruder_count` (`:14150`) and `flatten_bed_excluded_regions` (`:14158-14185`);
  - the public functions: `get_bed_excluded_regions_by_extruder` (`:14211-14243`), `get_bed_excluded_regions` (`:14271-14286`), `is_valid_bed_exclude_volumes_string` (`:14298-14317`), `has_bed_exclude_volumes` (`:14346-14355`) and `active_bed_exclude_volume_mode` (`:14364-14369`).
- **`ExclusionVolumeGeometry.cpp`:** `bed_exclusion_arrange_polygon` (`:70-83`). A convex region is kept exact, a concave one is replaced by its bounding box.
- **`PartPlate.cpp`:** `PartPlateList::preprocess_exclude_areas`'s resolved-region branch (`:7725-7761`). Each collision volume of the active groups becomes a `BedExclusionVolume<e>_<r>` blocker on every plate, with its z range and its extruder (-1 when Shared). It goes between the wrapping area and the legacy areas, which `plate_list::exclude_areas` already ports.

### Ares destination
- **`crates/ares-core/src/options/bed_exclusion.rs`**, public as `ares_core::bed_exclusion`:
  - `BedExcludeVolumeMode`, `BedExcludeRegion` (with `is_collision_volume`);
  - the five public functions above, over a key-to-token config with registry defaults: the A3b `DynamicConfig` made public. That is what upstream's `DynamicPrintConfig` overloads read, and it lets a test start from the registry defaults (`full_print_config()`) and set keys as upstream does.
  - `arrangement` gains `bed_exclusion_arrange_polygon`.
- **`plater::arrange_job::exclusion_items`** adds the blockers. Ares's `ArrangePolygon` already carries `is_bed_exclusion`, `bed_exclusion_extruder_id`, `z_min`, `z_max` and `has_z_range`, and the A1 kernel already honours them (`arrangement::bed_exclusion_applies`).
- **Prerequisite: three typed keys.** `bed_exclude_volumes` (String), `bed_exclude_volume_mode` (Enum) and `extruder_bed_exclude_volumes` (Strings) are in the registry but not in the typed `ProjectSettings`. `collect_config_entries` serializes only typed fields, so `DynamicConfig::new(settings)` would read their registry defaults and miss a project's value. A2b1 needs them typed in the printer options (`PrintConfig.cpp:1063-1089` defines them). Whether the config slice or A2b1 adds them is an open question.

### Included and deferred
- **Included:** all three modes of the resolution (they are pure functions of the config), the arrange polygon, and the blockers.
- **Deferred:**
  - A2b2, above.
  - `project_bed_material_keepouts` (`:14089-14109`), the belt projection: belts are deferred.
  - `append_bed_exclude_volumes` (`:14319-14344`), a GUI editing helper.
  - The `validate()` error keys (`:12475-12490`).
  - The G-code, brim and travel consumers (`GCode.cpp:6037`, `Print.cpp:4295`, `active_bed_exclusion_footprints`), which are slicing, not arrange.

### RED
- **Unit cases**, ported literally through the public API: `crates/ares-core/tests/bed_exclusion.rs`, citing `test_exclusion_volumes.cpp`.
  - The resolution: `:178`, `:252`, `:299`, `:319`, `:332`, `:350`, `:367`, `:382`, `:403`, `:417` and `:433`.
  - The arrange polygon: `:507`.
  - `two_extruder_config` (`:49-60`) and `rectangle` (`:30-40`) are copied as written.
  - Left out: `:96`, `:169` and `:269` test the `ConfigOptionPoints` / CLI-option layer; `:228`, `:452` and `:471` test validation and the legacy first-layer helper; `:105` and `:153` test belts; the rest is A2b2 or slicing.
- **Native parity case** in `plater_arrange.rs`: the probe above as a project edit (`bed_exclude_volumes` set in `project_settings.config`), arranged by the session and by Native `--arrange` (`assert_arranged_like_native`).
- **Expected before GREEN:** the unit cases fail on a `todo!()` skeleton, and the parity case fails because the session packs the pile onto the volume.

## A2b2: the plate outside check against bed exclusions

Every `file:line` is at `345efec64`. The session's tower path asks whether a plate holds an instance whole (`contain_instance_totally`). Upstream answers through `PartPlate::check_outside`, which tests the instance against the bed exclusions. Ares's `PlateList::outside` has neither exclusion branch, so the session refuses "arrange" with a prime tower beside a legacy area or a collision volume (`arranging with a prime tower beside a bed exclude area is not supported yet`). This slice ports the check and lifts the refusal.

### Upstream boundary
- **`PartPlate::check_outside`** (`PartPlate.cpp:4524-4625`), the non-sinking branch that runs when the plate box contains the instance box:
  - **Config-region branch** (`:4562-4601`). This is the branch the session takes, because the GUI has `m_plater`.
    - It reads extruder group 0 of `get_bed_excluded_regions_by_extruder` (A2b1) and skips collision volumes unless the mode is Shared.
    - Each region moves by the plate origin and is made counter-clockwise.
    - An instance is outside when all three hold: its 2D convex hull intersects the region (Clipper `intersection`), its box overlaps the region's z range, and `intersects_bed_exclude_region` holds.
  - **`m_exclude_bounding_box` fallback** (`:4603-4612`, boxes from `:626-643`): it runs when no config region was checked, as in the CLI. It tests the hull against each legacy-area box.
- **`ModelInstance::intersects_bed_exclude_region`** (`Model.cpp:3753-3836`), the yes/no path (`intersection_mesh == nullptr`), with its helpers from the anonymous namespace (`:3559-3750`):
  - footprint and clipping: `ExclusionPrismFootprintTriangle`, `cross_2d`, `clip_polygon_by_signed_distance`, `triangulated_prism_footprints` (over `triangulate_expolygon_2d`, `Tesselate.cpp:216`, beside the ported GLU kernel), `triangle_bbox_intersects_prism_footprint`, `clip_model_triangle_to_prism` and `polygon_3d_area2`;
  - the enclosed-prism fallback, `mesh_contains_point_linear`.
  - Every model part's triangles go through `instance matrix * volume matrix`. A clipped area above `1e-8` counts as a hit; otherwise a probe at the first footprint's centroid, mid z range, is tested for containment in the part's mesh.
- **The refusal** in `PlaterSession::arrange` (`plater/arrange_job.rs`) goes, together with its `has_bed_exclude_volumes` and legacy-area test.

### Ares destination
- **`crates/ares-core/src/plater/plate_list/`**: `PlateList::outside` gains both branches, so `contains_totally` and the tower estimate see the exclusions. It takes the resolved regions from the session's `DynamicPrintConfig`.
- **A new `crates/ares-core/src/options/bed_exclusion/` submodule** holds the intersection: the per-part test is `pub fn model_part_intersects_region(vertices, indices, matrix, &BedExcludeRegion) -> bool`.
  - This is the public surface the unit cases need, because Ares has no public `Project` constructor for upstream's `CubeModel`.
  - The instance loop over model parts stays crate-private.

### Included and deferred
- **Included:**
  - both `check_outside` exclusion branches;
  - the yes/no `intersects_bed_exclude_region`;
  - lifting the tower refusal.
- **Deferred:**
  - **A2b3, the per-polygon nozzle ids** (`ArrangeJob.cpp:53-103`): `set_arrange_polygon_bed_exclusion_extruders(_for_all_plates)`, `PartPlate::get_object_filament_extruders` (`PartPlate.cpp:3562-3654`), `get_effective_exclusion_filament_maps` (`:3540-3560`) and `bed_exclusion_extruder_for_filament` (`PrintConfig.cpp:14187-14209`, oracle `test_exclusion_volumes.cpp:486`). They need the plate filament map and the BBL-vendor test. Until then a non-Shared item keeps empty ids, so every nozzle's volume applies to it (`Arrange.cpp:131-133`), as upstream does while the mapping is unresolved.
  - The red intersection preview (`intersection_mesh`, `append_clipped_polygon`) and `intersects_bed_exclude_regions`, both GUI drawing.
  - `check_outside`'s sinking branch with exclusions (upstream's own TODO, `:4552`) and its IMEX branch (A2c).
  - Native's slicing refusal for an object inside a collision volume ("intersects a collision volume", return -63), which is slicing validation, not plater.

### Oracle and RED
- **Native `--arrange`**, where the CLI fallback and the GUI branch agree: rectangular legacy areas, and collision volumes no instance overlaps before the arrange (the CLI fallback does not see volumes). Each case uses the A3b two-filament rectangle tower (`plater_arrange/wipe_tower.rs`: `TWO_FILAMENTS`, `RECTANGLE_TOWER`, `PRIME_VOLUME_PURGE`, `NO_TOWER_BRIM`, the corner tower).
  1. A legacy `bed_exclude_area` rectangle on the bed's right side, clear of the cubes.
  2. A full-height `bed_exclude_volumes` box on the right side, clear of the cubes.
  3. The filament-2 cube starting over the legacy area: `check_outside` reports it outside, so the plate's filaments and the tower estimate change.
  - Today all three fail on the refusal. The probes in the RED commit pin each case's Native placement.
- **Unit cases**, through `model_part_intersects_region`, porting `test_exclusion_volumes.cpp` literally:
  - `:542`: crossing, above, outside, without the preview checks;
  - `:564`: the rotated and scaled instance;
  - `:574`: enclosed and above, without the preview checks.
  - `its_make_cube` (`TriangleMesh.cpp:902`) and the `rectangle`/`region` helpers (`:30-47`) are copied as written.
  - These are GUI-only, so upstream's unit tests are the oracle.
- **Expected before GREEN:** the unit cases stop at `todo!()`, and the parity cases fail on the refusal.
