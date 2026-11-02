# Classic unsupported wall-loop deferral

Status: approved Classic source boundary; commit this ARD before a frozen
public RED of at most three cases, Native first and strict bytes. Submit
the RED RR before any port; an unported prerequisite requires STOP.

Base: `5910df8eb0c2a9cab5958a1dcce7fdb34d075ecb`.
Upstream: OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Approved thin/narrow branches remain untouched; this proposal is not
rebased onto or falsely qualified by their earlier runtime evidence.

## Why this is next in source order

The next missing call after the Classic sandwich block is
`PerimeterGenerator.cpp:1965::defer_unsupported_loops`, before the
nonempty collection append. Its owning helper is1391-1455. The current
Rust `classic/perimeter_append.rs` implements the preceding wall order
and directly appends the collection, omitting this call and metadata.
The option exists only as registry metadata/default on this base;
`unsupported_wall_last` has no typed functional consumer and
`print_after_infill` is absent from the core loop vocabulary.

This is a port of upstream loop classification, attachment propagation
and its existing emission-pass contract, not an Ares-owned wall planner.
A metadata-only flag with no G-code consumer would not be a complete or
observable rewrite slice, so the exact consumer is named below.

## Exact owning source boundary

- `PrintConfig.cpp:5917-5925`: boolean `unsupported_wall_last`, default
  false. Reuse its existing registry definition; project its source value
  into region/perimeter options, without a new key, migration or policy.
- `PerimeterGenerator.cpp:1391-1398::is_unsupported_loop`: only a loop
  with nonempty paths all having `erOverhangPerimeter` qualifies. A mixed
  supported/overhang loop, bare path or multipath is not such a loop.
- `:1405-1420::defer_unsupported_loops`: disabled means no change. Stable
  partition anchored/nonqualifying entities first; stable-sort the
  unsupported tail by descending `inset_idx`, preserving equal-depth order.
- `:1422-1455`: collect existing anchored lines, visit the sorted tail in
  order, and propagate attachment through successive distance-index waves.
  A loop touches when any original polyline vertex has unsigned line
  distance **strictly less than**
  `1.5 * max(ext_perimeter_flow.scaled_spacing(), perimeter_flow.scaled_spacing())`.
  Newly reached lines anchor the next wave, not an invented nearest-path
  reorder. Unreached loops alone get `print_after_infill=true`.
- `:1965`: Classic call after configured wall reordering and before append.
- `ExtrusionEntity.hpp:474::ExtrusionLoop::print_after_infill`: false by
  default, retained on loop copies; not inferred again from labels later.
- `GCode.cpp:7742-7755` and8039-8053: ordinary non-infill-first perimeter
  pass, infill, deferred perimeter pass, then infill-first walls. Preserve
  first-layer and region/extruder semantics of these existing passes.
- `GCode.cpp:8889-8933::extrude_perimeters`: retain region selection;
  `defer_unsupported = !is_infill_first`; filter loop flag against
  `unsupported_loops_only`. Already-infill-first walls do not get a
  duplicate deferred emission. Preserve the source region-scoped printed
  prefix for any enabled inward wipe, rather than treating unprinted
  deferred loops as already printed support geometry.

## Rust destination boundary

All implementation stays inside existing core API modules:

1. `options/process_options/region_source.rs`, its `region_source/wire.rs`
   counterpart and `options/region_options.rs`'s resolved `RegionOptions`
   projection: expose the already registered boolean with the source false
   default. Do not duplicate registry keys, change unrelated presets or
   silently enable the option.
2. `project_slice/perimeters/classic/perimeter_append.rs` with a private
   child module for the helper: execute the source call after current
   reordering, using existing materialized entity paths/inset identity and
   nominal flow spacings from the aligned traversal record. Keep prior
   sandwich and accepted thin/narrow behavior unchanged.
3. `perimeters/classic/chained_loops/types.rs::ExtrusionLoop` (and its
   constructors/copy carriers): add the upstream false-default field,
   setting it only at the source helper. The shared neutral loop type may
   carry this vocabulary; that does not implement Arachne's producer call.
4. `project_slice/island_print_order.rs` and existing region/extruder
   emission adapters in `gcode_emit/motion/perimeter.rs`: implement the
   cited two perimeter passes around existing infill. Preserve original
   collection/region ownership and complete region perimeter context for
   existing wipe logic. No detached second slicing pipeline or new public
   scheduling abstraction.

Reuse the existing line-distance index only after checking that its
unsigned query and strict comparison match the cited source. Do not add
an alternate distance kernel, epsilon, sampled proximity rule or all-pairs
fallback. Stable order, wave boundaries and loop vertex operands are
non-negotiable; a faster but differently ordered reach algorithm is not
an acceptable substitute.

The current Ares option metadata, loop vocabulary and one-block perimeter
ordering are temporary compatibility shells for these upstream concepts.
Replace their missing source behavior; no legacy mode/fallback, public
API expansion, new crate, filesystem/UI/terminal access or native-only
runtime. Browser WASM, Windows, macOS and Linux remain Tier1 targets.

## Included and explicitly deferred

Include the Classic producer, option projection, loop field/transport and
its named G-code pass consumer as one end-to-end source slice. The inward
wipe printed-prefix bookkeeping at the consumer belongs to the same
selection contract; preserve existing wipe geometry rather than building
a different wipe algorithm. If that requires a distinct unported source
prerequisite, report it before implementation instead of omitting it or
waiving output.

Defer Arachne's separate call at `PerimeterGenerator.cpp:3108`; no Arachne
parity is claimed from a Classic fixture. If enabling this boolean on an
unported generator needs an admission restriction, agree it explicitly
at the public boundary before coding; never silently run the old behavior.
Also defer overhang-role generation/thresholds, support generation,
ordinary gap/fill geometry, sandwich permutations, seam placement,
wall-direction math, timing/cooling/M73, profile refresh/config-footer
fixes and unrelated path serialization. Reuse their current source data,
not a new feature-specific substitute.

## Required public RED and failure inventory

There is no existing fresh qualified witness for this proposal. Do not
claim one from older overhang or seam records. Proposed initial bounded
inputs, only after ARD/scope approval:

- Target: a Classic stepped ledge/island with at least one wholly
  overhang loop and an anchored wall stack, option enabled, wall order
  chosen so upstream's stable unsupported-tail/depth rule changes the
  public stream. Include an out-of-reach loop only when the same frozen
  input can prove the infill-before-deferred-loop consumer, not merely
  producer metadata.
- Disabled control: identical geometry, option false; preserve the
  ordinary complete output and avoid attributing base overhang/geometry
  failures to the new option.
- Attachment control: wholly unsupported loop that is within the exact
  source touch distance of an anchored wall (possibly transitively), so
  it remains before infill rather than receiving the unanchored flag.

First audit exact overhang-role/materialized operands and author the
smallest one candidate; freeze its geometry/settings/hash and intended
source premises before execution. The proposed RED budget is at most
three fixed cases, each Native2 fresh stable / validated API1 / CLI1,
with no retry, alternative producer, input search loop or debugger probe.
At most three frozen cases are approved, Native first and strict bytes;
freeze exact inputs before execution. No run is made in this ARD commit.
If Native does not exercise the exact producer/consumer, or an earlier
unrelated motion/role/config mismatch leads, preserve STOP and request
ruling rather than hunting geometry or patching an adjacent boundary.

Verify actual full ordered island/region extrusion passes, not only TYPE
labels or a raw metadata unit test. Wrong loop kind, mixed-role paths,
equal inset depth, transitive-only attachment, strict distance equality,
empty anchored set, infill-first/first-layer, multiple regions/extruders
and duplicate/omitted emission are failure cases to cover in the later
frozen breadth. Guard/premise tests do not automatically earn Native
parity credit.

Complete strict output keeps M73, ordered XYZ/E, travel/feed, TYPE/WIDTH,
wipe/retract, labels, config/footer, terminal whitespace and trailing
bytes. Only canonical generator lines normalize. External M73-command-only
diagnosis is permitted only with original-line maps, all removed records
and full contexts; never a comparator waiver. No coordinate/count golden,
loop sorting normalization or blanket reversal. Any surviving earlier
first difference remains NON-GREEN and is reported.

## Approval and verification sequence

The Classic ARD/source boundary and named G-code consumer are approved.
Commit this ARD first, then freeze and qualify at most three actual public
RED cases; submit the RR before any literal port. No implicit delegation
or unrelated queue write is authorized. Freeze Classic/wall-sequence/
overhang/region-infill/wipe before/after breadth before edits, require
zero prior-PASS to FAIL and report every flip and first remaining full/
non-M73 difference. Complete GREEN requires M73-inclusive whole streams;
COORD owns integration/FULL/closure. Preserve source/ELF/fixture/manifest
identities and every invalid attempt.

The approved narrow branch is not rebased or extended by this proposal.
This ARD sits on the explicitly requested main5910 base and has no
new runtime evidence, production/test edit or independent pipeline goal.
