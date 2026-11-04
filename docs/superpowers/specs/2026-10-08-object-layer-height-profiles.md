# Source-cited object layer-height profile rewrite slices

Status: Stage1 accepted; Stage2/automatic-adaptive/ZAA implementation not in scope.
No Rust edits, no Native/runtime attempts, no claimed RED/GREEN.

Architecture: [object-profile ARD](../../architecture/2026-10-08-object-layer-height-profile-boundary.md).
Priority: **range layer_height → imported painted profile → separate activeZAA**.
This is a libslic3r rewrite, not an independently designed Ares layering feature.

## Source ownership and present Ares seam

| Upstream owner | Behavior | Rust destination |
|---|---|---|
| Slicing.cpp:234-305 `layer_height_profile_from_ranges` | sorted overlap trimming, fixed first-layer interval, fill gaps, compress repeated z/h entries | `project_slice/profile.rs` |
| PrintObject.cpp:3805-3834 `update_layer_height_profile` | stored profile selection/validation, regenerate from ranges when required | same module; object data supplied by `planning.rs` |
| Slicing.cpp:807-870 `generate_object_layers` | profile interpolation at probes, shrinkage-scaled z, first-layer behavior, midpoint termination, preciseZ | existing `project_slice/layers.rs` |
| Slicing.hpp:45/52; Slicing.cpp `SlicingParameters::create_from_config` | fixed-first-layer predicate, uncompensated/compensated object coordinates and nozzle limits | existing `project_slice/parameters.rs` and submodules |
| PrintObjectSlice.cpp:817-830 `PrintObject::slice` | update profile before creating layers and slicing volumes | `project_slice/planning.rs::plan_project/plan_resolved_objects` |
| Format/bbs_3mf.cpp:2886-2942; :2092-2095 | range XML, one-based object-order association, ordered range map | existing `project/layer_config_ranges.rs` / project object association |
| Format/bbs_3mf.cpp:2824-2883; :2087-2090 | painted text profile import/object-order association | new focused `project/layer_height_profile.rs`, attached to `ProjectObject` |
| SlicingAdaptive.cpp:52-197; Slicing.cpp:309+ | mesh-face slope-derived automatic profile generation | deferred separate `project_slice` source port, not needed in stages1/2 |
| PrintObject.cpp:748-807; ContourZ.cpp:33-245 | activeZAA spatial extrusion contouring after layer generation | deferred separate source port, not `profile.rs` |

Current core is already close to the generate_object_layers consumer:
`layers.rs::generate_layer_pairs/profile_height/plan_print_object` accepts a
flattened alternating z/h profile. `planning.rs:51` always synthesizes the
fixed-only profile; `profile.rs` implements its compressed empty-range form.
Height-only ranges (no region override) are excluded from region-candidate
segmentation, and the raw range gate admits only those explicit-height ranges.
This is an equivalent mapping of PrintApply.cpp::generate_print_object_regions
(:958+, get_create_region :1000-1011) deduplicating PrintRegion by equal config:
height-only ranges yield the same PrintRegion, not a newly designed region scheme.
Pure None/region overrides/mixed ranges retain existing rejection; actual region-
range slicing remains a separate future stage.

`project/layer_config_ranges.rs:140-189` parses optional layer_height, replaces
identical range keys and lexicographically sorts ranges. Region candidates are
already resolved separately; **do not replace candidate/region machinery**.
`capabilities.rs:12-23` rejects imported painted-file presence and any participating
object range with layer_height; activeZAA is separately guarded.

The small internal profile-selection interface should accept existing object
range/profile data and slicing parameters and return the upstream flattened
profile. Keep selection/compression localized, not scattered among geometry and
G-code callers. Pass source object data at production planning time, not a fake
empty source in a compatibility wrapper. Existing internal planning tests can be
changed test-first to the same interface rather than preserving an obsolete API.

## Stage1 — object range layer_height (first implementation slice)

### Included source semantics

- Use the associated **source object's** range map, not project-global options,
  region layer candidates or independently guessed Z boundaries. Keep ranges
  in uncompensated object-relative profile coordinates; consumer applies upstream
  Z shrinkage. Top extent is object height, not absolute world Z.
- Match `layer_height_profile_from_ranges`: reserve linear storage, insert a fixed
  first-object-layer interval only when `first_object_layer_height_fixed()`;
  clamp each high to compensated object height per the source, advance low past
  prior accepted high, ignore spans failing `lo + EPSILON < hi`, fill unspecified
  gaps with configured layer_height and append through uncompensated object top.
- Preserve source `lh_append` approximate z/h equality and duplicate/third-same-
  height compression. Do not round z to G-code precision or deduplicate jumps
  merely because their z is the same: distinct h at the same z describes a step.
- Route resulting profile through current pair generation, extrusion layer
  identity, mesh slice_z, print_z, height, layer-specific flows and G-code timing.
  Remove the `layer_height` capability rejection only after independent Native
  witness proves full-path use. Range region options must still apply.
- Audit first-layer/raft predicate, min/max heights, negative object min_z and
  preciseZ interactions against source instead of assuming the current fixed-
  profile shell covers them. Any required correction is part of this source
  slice and must receive its own prewritten RED case.
- Archive XML/object association remains byte-input core parsing. No filesystem
  or printer adapter change, no geometry reimplementation.

### Deferred / explicit unresolved source boundary

Painted import/selection, automatic adaptive generation, smoothing/UI tools,
texture generation and activeZAA are deferred. Unrelated slicing behavior does
not get refactored. Existing public external-input limits remain.

Stage1 boundary: **mixed Some/None ranges retain
UnsupportedProjectFeature("layer_config_ranges")** because the absent option
is directly dereferenced upstream (UB). No implicit default or UB imitation.
Pure None range collections preserve their pre-existing region-only behavior.
Stage1 consumes explicit-height collections only; this is a narrowed existing
unsupported boundary, not a new validation guard. Source property access is
PrintConfig.hpp::ModelConfig::option forwarding to DynamicPrintConfig::option.

Native range-profile code dereferences `option("layer_height")`; Rust stores an
Option because region-only ranges are accepted by its archive parser. Before
implementing, establish the admitted Native normalization/valid-input path for
mixed height-bearing and region-only ranges. Do not invent an implicit default,
broadcast missing heights or copy a Native null dereference. Start independent
fixtures with legal explicit layer_height on every participating range, plus
no-range controls. Mixed-range semantics must either receive a source-defined
RED/GREEN witness before accepting them or be called out as an unresolved scope
blocker; removing a blanket gate cannot silently create unsupported acceptance.

### Test-first RED construction

1. Preserve a known complete seededNative FDM 3mf control (box, stable single
   nozzle/profile, no activeZAA), bind input/Native ELF/source/seed/policy hashes.
2. Add only upstream `Metadata/layer_config_ranges.xml`, using the existing
   parsed one-based object-order association. Primary witness: a tall box with
   an interior interval at a legal height different from nominal, e.g.1..3mm
   at0.1 versus nominal0.2. Actual min/max/first-height values must be validated
   against the chosen loaded Native config; example is not an oracle assertion.
3. Native must successfully slice and emit complete output. Inspect Native
   layer Z/height list and exported range association to prove interval use,
   not just config presence. Current Ares must independently return named
   UnsupportedProjectFeature("layer_height") on that identical input. Commit
   runtime RED fixture/test/provenance **before** production patch.
4. Additional prewritten independent Native witnesses exercise overlapping
   sorted ranges/duplicate range key, sub-EPSILON/truncated ranges, first-layer
   overlap, default gaps, top clamp, object min_z/shrinkage, preciseZ, multi-
   object separation and non-height range region behavior as source permits.
   Do not predeclare unsupported Native combinations as output oracles. Native
   crashes stay UB_ADJACENT; timeout stays INCONCLUSIVE, neither becomes PASS.
5. Patch only profile owner/planning data flow and the proved gate. Run selected
   integration/parity tests through cargo nextest, not cargo test. GREEN requires
   complete emitted byte equality allowing only the whole generator line; layer
   schedules are a diagnostic artifact, not a weakened replacement comparator.
   Retain fixture, command/status, application proof, raw streams and hashes.

### Exit criteria

Primary and legal boundary witnesses are strict full-byte GREEN; unmodified
fixed-height controls retain exact bytes. Native-rejected inputs remain explicit
non-PASS. Fresh same-round rebased base/candidate seeded full gate, targeted
cases, workspace nextest, cargo clippy, cargo fmt, browserWASM check, and the
official gate pass/are honestly reported. No ignored tests, value filters or
retries-to-PASS.

## Stage2 — imported painted layer_height_profile (separate slice)

Current `project/load.rs:66-70` records only case-insensitive file presence in
`ProjectDocuments`; presence is not parsed per-object data and currently blocks
all planning even when there is no usable profile. Replace this compatibility
shell with the cited import owner and per-object profile association.

Included: parse admitted `Metadata/layer_heights_profile.txt` records
`object_id=N|z;h;...`, resolve **one-based model object order**, reject/ignore bad
records exactly according to proved upstream import/error behavior, retain the
object profile as model data, and port `update_layer_height_profile` selection:
copy stored vector, invalidate odd size or top mismatch >1e-3, regenerate from
ranges if empty or first h differs from first-object-layer-height. This is
upstream documented selection, not an Ares legacy fallback. Importer accepts
only even value count >4 although exporter accepts >=4; preserve the actual
consumer rule and test it rather than conflating the two. No UI texture/smoothing
or source-adaptive slope generator in this slice.

Before code, Native RED must demonstrate a valid painted profile with >=6
values and matching first height/top, output visibly different from range-only
control, while Ares returns named UnsupportedProjectFeature("layer_height_profile").
Then write all invalidated-profile/range-regeneration and multi-object association
cases before patching import/selection. Remove only this gate when legal cases
match complete bytes. Profile presence/absence old tests are replaced test-first
with imported semantic records, not silenced. Same acceptance gates as stage1.

## SlicingAdaptive assessment — adjacent, not necessary for consuming profiles

`prepare` transforms the raw object mesh using its first instance/handedness,
computes f32 face Z spans and normalized slope, lexicographically sorts faces.
`next_layer_height` interpolates quality-to-deviation, tests active/upcoming
facets using a monotonic cursor, slope metric and min/max clamps. Automatic
`Slicing.cpp::layer_height_profile_adaptive` builds/smooths an output profile.
`PrintObject::update_layer_height_profile` does **not** automatically call it
(the attempted call is commented; actual regeneration calls from_ranges).
Therefore importing ranges/painted profiles must not introduce adaptive quality
parameters, speculative mesh scanning or a self-designed Ares algorithm.
Automatic profile creation remains a future source-cited port if requested.

## ZAA2.4.2 assessment — confirmed libslic3r, independent future rewrite

All four researched upstream files match the admitted seeded2.4.2 source by
SHA256 (ARD). Thus activeZAA is a real libslic3r port gap, not GUI-only behavior,
not an invented Ares feature, and not the SlicingAdaptive layer-height algorithm.
`need_z_contouring` checks region zaa_enabled; `contour_z` transforms raw mesh,
builds `sla::IndexedMesh`, applies ground-level offset and contours layers from
index1. The SLA-namespaced mesh helper is reused here for **FDM** path contouring.
`ContourZ.cpp` samples path segments at0.1mm, ray-hits upward from mesh slice_z,
changes per-point spatial z, clamps against min_z/layer height, corrects perimeter
slope and handles top-solid/ironing/perimeter collections. Consequently this
needs source-owned mesh-ray query and 3D extrusion path/G-code integration,
not just changing LayerPair or toggling zaa_min_z. Existing box-inactive proof
cannot justify deleting the gate for sloped objects or modifiers.

A future independent spec should cite ContourZ.cpp + SLA/IndexedMesh ray query +
spatial ExtrusionPath owners and destination geometry/path seams, with a sloped
activeZAA Native RED. No ZAA production code or capability removal in stages1/2.

## Runtime discipline

If a Native failure requires debugger evidence, use `nix shell nixpkgs#gdb -c gdb ...`
on the **existing admitted binary**, without rebuilding OrcaSlicer for diagnosis.
Current document is source-read evidence only, not runtime proof.
