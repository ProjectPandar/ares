# Object layer profiles and range-aware volume slicing

Upstream: checked OrcaSlicer revision `8500fcdcc`.
Two independent RED/GREEN slices below.

## Boundary and ordering decision

Port imported `ModelObject::layer_height_profile` selection first, then the
remaining range-aware `PrintObject::slice_volumes` admission and routing.
Do not create an Ares adaptive-layer algorithm or a second region registry.

Main already ports explicit-height ranges into profiles in
`project_slice/profile.rs`, profile interpolation and precise-Z adjustment in
`layers.rs`, and range-dependent PrintRegions in `effective_config`,
`volume_regions` and `region_slices`. These are reusable upstream rewrite
modules; their remaining partial assumptions are compatibility scaffolds.

The dependency is semantic: `PrintObject.cpp:3805-3833` selects a stored valid
profile before invoking `Slicing.cpp::layer_height_profile_from_ranges`.
The latter dereferences every range's `layer_height` option at `Slicing.cpp:249`.
A valid stored profile therefore supplies a defined route for region-only
ranges without inventing a default height or copying upstream UB. Region-range
selection subsequently uses actual generated layer slice Z, not profile knots.

Current `capabilities.rs:12-14` rejects profile-file presence globally. Its
`:16-27` guard checks mixed Some/None range heights, not scalar `layer_height`.
`raw_intersections.rs:272-291` rejects missing range height and every region
override except `wall_loops`. The spiral range guard at `capabilities.rs:29-37`
has a separate upstream assertion boundary; ZAA is unrelated to this slice.

## Upstream owners and Rust destinations

| Upstream file/function and lines | Behavior | Rust destination |
|---|---|---|
| `Format/bbs_3mf.cpp:215,2824-2883`, `_extract_layer_heights_profile_config_from_archive` | `Metadata/layer_heights_profile.txt`, newline records, `object_id=N\|z;h;...`, ordinal association, double values | New `project/layer_height_profiles.rs`; minimal `project/load.rs` and module registration |
| `Format/bbs_3mf.cpp:2087-2090` | Associate by original one-based ModelObject ordinal | Object-owned profile in `project/domain.rs`; retain through clone/plate selection |
| `Format/bbs_3mf.cpp:7482-7503` | Export ordinal and six-decimal semicolon profile | Authored fixture contract, not a new production exporter |
| `PrintObject.cpp:3805-3833`, `update_layer_height_profile` | Copy stored vector; discard odd length or top mismatch >1e-3; regenerate when empty or first height differs exactly | `project_slice/profile.rs` selection, called by `planning.rs` |
| `Slicing.cpp:232-304`, `layer_height_profile_from_ranges` | Lexicographic ranges, earlier-range precedence, fixed first layer, gaps, compression, compensated clipping/uncompensated tail | Reuse existing `profile.rs`; change only a separately demonstrated discrepancy |
| `Slicing.cpp:807-866`, `generate_object_layers` | Double interpolation, scaled Z knots, layer midpoint termination, fixed first layer, precise-Z adjustment | Existing `layers.rs`; no independent sampling loop |
| `PrintObjectSlice.cpp:823-829` | Select profile, create layers, then slice volumes | Existing `planning.rs` → `state.rs` handoff |
| `PrintApply.cpp:342-395`, `LayerRanges` | Continuous range partition, overlap/gap/epsilon semantics | Existing `project/effective_config/layers.rs` |
| `PrintApply.cpp:958-1027`, `generate_print_object_regions` | Config-distinct object-wide regions, volume-major mappings, volume occupancy | Existing region graph/candidate resolution and `volume_regions` |
| `PrintObject.cpp:3664-3705`, `region_config_from_model_volume` | Object/volume/material then range overrides, feature masks and normalization | Existing `options/region_options/merge.rs`; verify its existing range-last order |
| `PrintObjectSlice.cpp:100-136`, range `slice_volume` overload | Filter float Z by half-open occupancy ranges; scatter results into aligned layer slots | Private new `project_slice/raw_intersections/ranges.rs`, minimal parent wiring |
| `PrintObjectSlice.cpp:148-228`, `slice_volumes_inner` | Each volume's active ranges; model/negative/modifier ordering; single-range shortcut | Same private range owner, reuse existing mesh intersection and volume IDs |
| `PrintObjectSlice.cpp:244-266`, `layer_range_first/next` | Epsilon-sensitive first/next region selection | Reuse existing `region_slices/range_lookup.rs` |
| `PrintObjectSlice.cpp:1149-1199`, `slice_volumes` | Allocate all regions, slice by volume, route by range, trim empty top layers | Existing `region_slices` / state; no parallel replacement pipeline |

All paths above are under `OrcaSlicer/src/libslic3r` or
`crates/ares-core/src`, respectively. No core filesystem/Native/UI calls,
public pipeline API, external crate or runtime policy switch is needed.

## Slice P: stored profile import and selection

Included:

- Import structurally valid records with more than four, even-count finite
  values; preserve double values and pair order. The upstream importer rejects
  four-value records at `bbs_3mf.cpp:2867-2871` although its exporter accepts
  them; do not infer import semantics from the exporter.
- Associate before plate filtering using original object ordinals, never ZIP
  model IDs or the first object as a stand-in for all objects.
- Use a profile ending at the uncompensated object top within the upstream
  tolerance. Preserve exact first-height comparison and legitimate repeated Z
  transition knots; do not sort or resample an authored vector.
- Apply the actual selected profile to existing layer generation, including
  shrinkage-Z interpolation and existing precise-Z correction.
- Profile top/first-height mismatches follow upstream profile regeneration,
  through the one existing range-profile function. This is the cited upstream
  selection rule, not a legacy fallback.
- Remove the presence-only profile guard after public Native-backed RED. Keep
  unrelated gates unchanged in this first slice. Replace the obsolete opaque
  presence contract only where the newly imported data is being asserted.

Boundary cases: empty file, four-value record, duplicate ordinal, unknown
ordinal, odd count and malformed numeric input require an import preflight
against Native before choosing their exact public diagnostic behavior. Native
import diagnostics must be recorded; do not silently drop data or invent an
Unsupported gate to claim parity. Any ambiguity is reported before GREEN.

No-range inputs continue to call the existing range-profile generator and
must retain identical raw G-code. Profiles on distinct objects remain distinct.

## Slice R: range configuration and volume routing

Included after P:

- Defined Native paths with explicit height on all ranges, or a valid stored
  profile which prevents the missing-height dereference. Test mixed height and
  region-only ranges only on the latter path. Retire the mixed-height rejection
  for that source-proved path; do not synthesize missing heights.
- Admit region changes beyond `wall_loops`, initially using independent
  witnesses for `sparse_infill_density`, extrusion width and wall speed. Existing
  typed RegionOptions and downstream consumers remain the owners. No new
  whitelist/Unsupported gate is introduced.
- Reuse object-wide config deduplication and range mappings. Apply overrides to
  model parts with upstream precedence; parameter modifiers inherit their
  parent and own config rather than treating range config as modifier config.
- Supply each existing volume occurrence's active ranges to raw slicing;
  filter the actual f32 slice heights by `[min_z,max_z)`, preserve empty slots
  when scattering, and preserve original volume identity/order. Do not collapse
  to `regions[0]` or recompute a schedule per region.
- Preserve and verify existing epsilon first/next routing and top-empty-layer
  handling under a nonuniform schedule. No change to the comparison contract.
- Remove raw `validate_layer_ranges` rejection once its owning behavior has a
  real RED/GREEN. Make parent-file changes minimal; new private files ≤400 LOC.

Missing-height ranges when no valid stored profile survives selection reach
upstream UB (`Slicing.cpp:249`). They are not acceptance witnesses. The prior
ARD's mixed-range restriction is not globally declared solved; its remaining
UB boundary needs an explicit decision before retirement. No new rejection,
silent acceptance or guessed normal-height rule is introduced here.

## Public E2E RED design

Inputs are authored 3MF projects, committed with provenance and input hashes;
Native output and diagnostics are not committed. Use the
existing CLI `OrcaRunner::slice_case` cache and public core validated byte API,
plus the CLI where the case tests an import/validation result. Each execution
uses a fresh Native cache and artifact root and required dependency policy.
Compare complete ordered G-code bytes with only the existing
generated-by normalization, never a footer/polygon-only comparison.

| P controls | Independent observable |
|---|---|
| P0: no profile | Inactive raw-output control |
| P1: linear ramp with ≥3 pairs | Native accepts; Ares currently rejects profile; full stream and all Z/HEIGHT entries |
| P2: stepped/repeated-Z profile | Knot transition/interpolation, first differing physical layer |
| P3: two objects with different profiles | Original ordinal association, merged layer schedule |
| P4: selected plate with an earlier unselected object | Association before plate filtering, complete selected output |
| P5: wrong top; P6: wrong first height | Native range-profile regeneration, complete byte equality |
| P7: precise-Z and non-unit shrinkage (deferred after RED) | Independent no-profile Native11/Ares12 finding; separate slice after P merge |

P1/P2 establish RED before production import/selection changes. A Native
rejection or producer instability is reported, not hidden by changing context.
Secondary cases whose existing Ares paths are independently unsupported must
be reported before widening scope; do not make them pass by weakening tests.

| R controls | Independent observable |
|---|---|
| R0: existing height-only and wall_loops controls | No regression in existing strict comparisons |
| R1: explicit-height ranges change sparse density | Real raw admission RED, distinct in-range paths |
| R2: explicit-height width and speed ranges | Native E/F and path behavior across exact boundary |
| R3: overlap/gap/epsilon boundaries under P profile | Region first/next cursor and physical layer identity |
| R4: valid profile + mixed height/region-only ranges | Native avoids missing-option UB, real mixed guard RED |
| R5: multipart and modifier crossing a Z boundary | Per-volume occupancy and parent mapping, whole output |
| R6: range value conflicts with volume value | Upstream range-last precedence, whole output |

P and R each have a separate RED then GREEN commit. Main GREEN
is not established by a private layer-vector unit test or by Ares success
alone; Native official and seeded must both demonstrate their real result.
Existing pure Rust assertions are retained unless their rejected feature is
the specific supported behavior; then migrate to positive E2E assertions and
list every expectation change.

## Included/deferred and acceptance

Deferred: mesh-derived `SlicingAdaptive` generation and GUI painting/smoothing;
active ZAA/ContourZ; spiral with multiple region ranges (upstream
`PrintObjectSlice.cpp:213` assertion); new support/raft algorithms, painting,
tool mapping and unrelated capability guards. Existing implementations may be
exercised as controls, but no guard is replaced by a new Unsupported guard.
This proposal does not assert universal layer-range option completeness.

For each slice: official+seeded directed RED/GREEN; all existing height/range
E2Es; default-env workspace; strict all-target/all-feature clippy; fmt check;
core/vgcode/wasm wasm32 check. Run full option_coverage both modes if any
inventory/plan/sweep registration is changed. Audit raw-output changes, with
non-profile/non-range controls byte-identical and any other changes reported.
Fresh canonical full requires the oracle to be available; no full or
missing-oracle-dependent directed run otherwise.
