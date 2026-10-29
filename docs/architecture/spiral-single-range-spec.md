# SPEC: release single/multi-range spiral slicing

Status: SPEC REVIEW-REQUEST preparation; documentation and Native observations
only. No product edit or guard removal is authorized by this note.
Owner amendment: the official release uses NDEBUG; the multi-range branch is a
defined literal-port target, not UB merely because a debug assertion rejects it.
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

The official 2.4.2 release compiles that assertion out under NDEBUG. Entering
the otherwise branch then executes the ordinary defined ranged-slicing code;
the assertion itself is not an undefined read. The owner explicitly approves
using deterministic completed Native release runs as authority for R1-like
multi-range cases as well as the single-range path. Debug-assert behavior must
not become a new release-only rejection in Rust.

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
  unfiltered upstream `LayerRanges` count for mixed inputs. The older
  `2026-10-08-object-layer-height-profile-boundary.md` decision5 equates
  equal PrintRegion configurations with unsplit candidates; the source
  distinguishes those two counts. This RR requests correcting that assumption
  by retaining source ranges, not building a second approximate counter.
- `project/region_graph.rs::object_region_graph`: bounds/membership and
  configuration reuse already exist; no parallel registry is proposed.
- `project_slice/slicing_mode_intersections.rs::model_part_threshold`,
  `spiral_bottom_threshold`, `apply_layer_mode`, and `largest_contours.rs`
  already own base-below/PositiveLargestContour processing. Reuse and verify
  these source-shaped owners rather than design another spiral algorithm.
- `project_slice/raw_intersections/ranges.rs` remains the ranged-volume owner.

After review and a Native-backed public RED, admit Native-completing,
deterministic cases such as C0/R1/R2 rather than rejecting all range overrides.
Port the branch selected by the **source-faithful global range count**:

- Single range: preserve matching-volume membership, region lookup, the bottom
  cutoff and PositiveLargestContour upper layers already owned by the existing
  modules.
- Multiple ranges: do not invoke the single-candidate spiral threshold or panic
  because multiple candidates exist. Keep the configured base slicing mode for
  all intersections, build this volume's occupied ranges, and use the ranged
  slice-volume path literally. For R1 only one range contains the volume, but
  global count2 still selects this multi-range branch.

Determine source cardinality before any height-only compatibility filter; do
not count just nonempty spans or distinct region configurations. Retire that
filter as part of the source assignment rewrite, reusing region configuration
deduplication and bbox membership downstream. Review the older height-profile
boundary decision5 accordingly and retain height-only/mixed-range regression
controls. One canonical assignment owner replaces the temporary shell; no
duplicate planner, public flag or shape/input whitelist is authorized.

Included: literal single and release multi-range branches, range membership and
per-layer configuration, the source-derived schedule, regular stock bottom/
positive-largest single-range transition, and consistent API/CLI behavior.
Defer Native-rejected or nondeterministic cases for evidence/ruling rather than
invent acceptance. Missing-height profile-regeneration policy, adaptive height
generation, unrelated vase emission and scarf are unchanged. No default-height
fabrication; core stays portable with no Native invocation or file I/O.

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
  GDB must confirm the multi-range branch; literal release behavior, not the
  compiled-out debug assertion, owns this active-override witness.
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

Send this amended spec plus probe receipts as REVIEW-REQUEST **before product
edits**. Report Native rejection/crash/nondeterminism without retries to
manufacture a pass. GDB must identify the selected branch; a completed R1 is a
multi-range release witness, never mislabeled single-range. R2 is independently
an ignored-override admission witness, not active speed-override behavior.

A later authorized RED uses the public byte API and CLI on identical C0/R1/R2
inputs; Native must complete first in both official and seeded modes. Preserve
strict complete ordered-byte comparison with only the existing generator-line
normalization. In particular preserve R1's actual layer count/Z schedule,
region speed and motion, not C0's schedule or a guessed one-range result.
No tolerances, assertion waivers or expanded normalization. Refuse only the
Native-rejected/nondeterministic boundary demonstrated for this slice; leave
other independent guards and invalid-input validation untouched. Future GREEN
requires both-mode directed parity, required-oracle non-parity, fmt/clippy and
a clean source-cited diff. Name inherited failures rather than calling them green.
