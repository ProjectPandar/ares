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
