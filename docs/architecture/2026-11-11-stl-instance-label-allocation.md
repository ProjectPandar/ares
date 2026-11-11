# Direct-STL instance label allocation

## Upstream boundary

This slice rewrites the `ModelVolume` default-member allocation contribution to
`ModelInstance::get_labeled_id` for a single direct-STL CLI import. Source references
are to OrcaSlicer revision `345efec64c7b95a05ce5cf96df0dde66a158835c`.

`src/libslic3r/ObjectID.hpp:76–87,100–111` allocates one process-global ObjectID
for an ordinarily constructed `ObjectBase`, including `ObjectWithTimestamp`.
The integer-argument deserialization constructor does not allocate an ID.
`Model.hpp:758,802–805` makes ordinary `FacetsAnnotation` construction allocate
through that base.

The direct-STL construction sequence is:

| Construction | Source | Allocations |
|---|---|---:|
| Outer CLI `Model` and wipe tower | `src/OrcaSlicer.cpp:1730,1757`; `Model.hpp:1593–1610,1695–1704,1753` | 2 |
| `Model::read_from_file` local model and wipe tower | `Model.cpp:331,353–354` | 2 |
| Object, config, layer-height profile | `Format/STL.cpp:39`; `Model.cpp:631–637`; `Model.hpp:582–588` | 3 |
| Volume, config, four existing facets, eight texture-displacement facets | `Model.cpp:1402–1404`; `Model.hpp:903–935,1205–1220` | 14 |
| Default instance | `Model.cpp:506–507,805–816,1554–1562` | 1 |

The first instance therefore receives `2 + 2 + 3 + (6 + 8) + 1 = 22`.
The eight `texture_displacement_facets_0` through `_7` members allocate even
when the input contains no texture-displacement data. STL leaves `loaded_id`
zero, so `Model.hpp:1433–1443` returns the instance ObjectID. The header
collects and emits that label in `GCode.cpp:4224–4239`.

## Rust destination and invariants

`crates/ares-core/src/project/domain/profile_input/stl.rs`,
`Project::from_stl_with_profiles`, represents this sequence with an allocation
count. Include the eight default facet members in that count and continue using
one label for both `ProjectInstance` and `PlateMetadata`. Do not alter header
formatting or substitute an output-only label.

This count remains a temporary compatibility shell around upstream model
construction, not an independent Ares identity design or a general ObjectID
allocator. It is scoped to the existing single-STL import. No global mutable
counter, platform-specific behavior, I/O, or fallback belongs in core.

## Included and deferred behavior

Included: the first direct-STL instance's label, its plate identity, and the
upstream default-member allocation explanation.

Deferred: generic ObjectID lifetimes, multiple-file imports, cloning, GUI
construction order, texture-displacement geometry, 3MF loaded labels, and
unrelated configuration, geometry, or timing differences. The additional facet
members affect allocation count here; their mesh processing is not introduced.

## Verification boundary

The public profile-slicing test slices the same positioned cube as Native and
compares the runtime Native label. Remove its stale literal-14 premise rather
than replacing Native with another literal oracle. Retain raw API output beside
the input, profiles, and Native output so the result can be repeated and audited.

After the count correction, compare the complete Native, API, and CLI byte
streams for this STL without filtering, normalization, or padding. If all match,
the case can serve as a complete-parity witness. Otherwise retain the first
differing byte and line, identify the owning upstream function, and leave that
adjacent behavior outside this slice. A matching label alone does not establish
whole-output parity.
