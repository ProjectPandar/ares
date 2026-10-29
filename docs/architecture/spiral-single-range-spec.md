# SPEC: source-owned single-range spiral admission

Status: SPEC REVIEW-REQUEST preparation; documentation and Native observations
only. No product edit or guard removal is authorized by this note.
Base: `2859763166e4a16e4aa4db5bc01380f9316d414f`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Independent of the scarf policy; no scarf stack is required.

## 1. Exact upstream branch contract

Owner: `src/libslic3r/PrintObjectSlice.cpp::slice_volumes_inner:148-228`,
particularly `:194-222`.

1. Sort volumes by source ID; process only volumes requiring slicing.
2. Test the **entire supplied `layer_ranges` vector's size**, not the number
   of source XML entries, distinct PrintRegions, or ranges intersecting this
   particular volume.
3. If `layer_ranges.size() == 1`, take its sole range. Slice the volume only
   when `layer_range.has_volume(model_volume->id())` is true.
4. Inside that branch, **only a model-part volume with `spiral_mode=true`**
   switches `params.mode` to `PositiveLargestContour`. Negative and modifier
   volumes do not receive this switch.
5. Read the matching volume region's configuration, not a global default.
   Initialize `slicing_mode_normal_below_layer` from `bottom_shell_layers`;
   increment while below `zs.size()` and
   `zs[index] < bottom_shell_thickness - EPSILON`. Do not clamp away the
   source count when it exceeds the number of layers.
6. `params_base.mode_below` is initialized from the configured base slicing
   mode and copied into `params`. Thus stock `slicing_mode=regular` gives
   **regular bottom layers followed by PositiveLargestContour**. Literally,
   EvenOdd/CloseHoles retain their respective base mode below the cutoff;
   the comment saying regular bottom is not permission to overwrite that
   actual field. Keep the source f32 Z comparison and EPSILON boundary.
7. Otherwise the source reaches `assert(!print_config.spiral_mode)` at
   `:213`, then constructs this volume's active slicing intervals and calls
   ranged `slice_volume`. Generated range vectors are nonempty, so this
   otherwise branch is the multi-range path in the admitted source graph.

Multi-range spiral remains unsupported. An assertions-disabled Native build
may complete on that path; completion does not establish defined single-range
spiral behavior or authorize enabling that path in Rust.

## 2. How upstream obtains one versus multiple ranges

`src/libslic3r/PrintApply.cpp::LayerRanges::assign:342-395`:

- Input map order is lexicographic `(low, high)`; earlier coverage trims later
  intervals. Start `last_z=0`; consider an entry only when its upper bound is
  greater than `last_z`.
- Clamp its low bound to zero. Insert a null-config default gap only when
  `low > last_z + EPSILON`; append the configured span only when
  `high > last_z + EPSILON` after that gap handling.
- If nothing was emitted, create one null-config `[0, DBL_MAX]` range.
- If the last emitted range is null-config, extend it to `DBL_MAX`.
- Otherwise append a null-config tail `[last_z, DBL_MAX]`.

Consequences that the admission check must preserve:

- One effective configured finite span normally yields **at least two**
  ranges: the configured span and default tail. A lower default gap can make
  three. Covering the whole physical model does not remove the tail.
- Even a configured span ending at `DBL_MAX` still takes the append-tail
  branch. Do not replace this with an invented full-height special case.
- Ignored intervals can leave the single default range. For example a range
  `[0, 0.00005]`, narrower than source EPSILON `0.0001`, emits no span.
  A non-default option in that ignored range is an **admission witness**, not
  evidence that an active override changes the single-range spiral geometry.

`generate_print_object_regions:958-1027` copies one layer-range record per
`LayerRanges` element (or verifies/reuses the same shape). Volume bounding-box
membership populates each record but does not erase out-of-model tail records.
`get_create_region` deduplicates PrintRegion configurations; it does **not**
coalesce the layer-range vector. Neither `all_regions.size()` nor per-volume
active-interval count substitutes for `layer_ranges.size()`.
`Print::apply` constructs this owner directly from the model object's raw range
map at `PrintApply.cpp:1727-1739`.

Archive ownership: `Format/bbs_3mf.cpp:2886-2939` loads range options;
`:2092-2095` associates them by **one-based model-object ordinal**, not 3MF
resource ID. The probe's sole model object uses ordinal1 even though its
model-settings resource ID is2.

## 3. Existing Rust boundary and proposed slice

- `crates/ares-core/src/project_slice/capabilities.rs`: second rejection
  currently checks spiral mode plus any raw non-default region override.
  It does not distinguish the actual source range vector's cardinality.
- `project/effective_config/layers.rs::layer_candidate_ranges`: existing
  assignment shell includes the source gap/tail logic, but deliberately
  skips height-only entries. Its length cannot be assumed identical to the
  unfiltered upstream `LayerRanges` count for mixed inputs.
- `project/region_graph.rs::object_region_graph`: bounds/membership and
  configuration reuse already exist; no parallel registry is proposed.
- `project_slice/slicing_mode_intersections.rs::model_part_threshold`,
  `spiral_bottom_threshold`, `apply_layer_mode`, and `largest_contours.rs`
  already own base-below/PositiveLargestContour processing. Reuse and verify
  these source-shaped owners rather than design another spiral algorithm.
- `project_slice/raw_intersections/ranges.rs` remains the ranged-volume owner.

After review and a Native-backed public RED, the candidate change is to reject
this guard's spiral cases when the **source-faithful global range count is
multi-range**, while allowing a proven single-range case through the existing
literal branch. Determine cardinality before any height-only compatibility
filter; do not count just nonempty spans or distinct region configurations.
Exact factoring of the existing assignment owner requires implementation
review; no duplicate planner, public option or heuristic is authorized here.

Included: the source single-range branch, matching per-volume region selection,
regular stock bottom/upper positive-largest transition, and consistent API/CLI
admission. Deferred: multi-range spiral, missing-height profile-regeneration
policy, adaptive height generation, unrelated vase emission and scarf changes.
The missing-height guard is unchanged; no scalar/default-height fabrication.
Core remains byte-oriented and portable, with no Native invocation or file I/O.

## 4. Native-first probe plan (CPU18 only)

Source fixture: `tests/parity/spiral-hole-projected-split/spiral.3mf`,
SHA256 `7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27`.
Stock MK4, single cube-with-hole object, spiral1, regular slicing, layer height
0.2, three bottom layers, zero bottom-thickness extension. Preserve every ZIP
member payload; append only `Metadata/layer_config_ranges.xml` for ordinal1.
All probe ranges include `layer_height=0.2`, avoiding missing-height UB.

- C0: unchanged no-range control; source predicts one default range.
- R1: one non-default `outer_wall_speed=17` override over `[0,1000]`, covering
  the object. Source predicts configured range plus default tail: **two**.
  Observe official/seeded exits and output, but do not label any successful
  release execution of this multi-range assert path as legal single-range.
- R2: the same non-default option over `[0,0.00005]`; source predicts one
  default range because the interval is ignored. Explicitly label this as
  an ignored-override admission case, not active speed-override parity.

Run official and seeded serially with fresh output directories, record complete
argv/exits/stdout/stderr, input and output hashes, and three repeats per case.
Normalize only the existing generated-by timestamp suffix for repeat digests.
Check CONFIG_BLOCK_END, spiral footer settings, layer/Z activity and equality
against C0 for the ignored case. Retain all observations outside the repository.
Use the existing pinned producers; no upstream source edits or new oracle build.
If needed, use matching seeded debug information to observe the actual global
vector size and mode/cutoff; keep official observation limits explicit.

## 5. Review and implementation gates

Send this spec plus probe receipts as REVIEW-REQUEST **before product edits**.
Report Native rejection/crash/nondeterminism without retries to manufacture a
pass. Native success is necessary but insufficient: the source branch must be
single-range. If only the ignored case is safe, state that limited reachability
and do not claim support for an active range override.

A later authorized RED uses the public byte API and CLI on the identical input;
Native must complete first. Strict complete ordered-byte comparisons retain the
existing generator-only normalization, with no tolerances or assertion waivers.
Keep multi-range rejection tests and unrelated boundaries. Future GREEN requires
both-mode directed parity, required-oracle non-parity, fmt/clippy and a clean,
source-cited diff. Any inherited failures must be named rather than called green.
