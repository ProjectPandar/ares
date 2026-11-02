# Classic unsupported wall-loop deferral

Status: approved Classic ARD committed before fixture work as
`9f2e1658197effcb31bb554a38ae23ac97d56954`; frozen Native-first public
qualification is complete (two strict REDs and one complete control PASS).
The frozen RED `aeff66b69d8acee8310cb024cc04353983c9f10b` and doc-only
nullopt boundary amendment `d6a0309947b37e4ee66bea6a318434c41c0738f3` are
approved. The literal Classic port is strict GREEN for both enabled cases
and the disabled control, with frozen paired 298-test scope and 0 P→F.
The entire `wipe_inward` slice remains explicitly deferred.

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
  duplicate deferred emission. Port this consumer exactly as its
  `wipe_support == nullopt` branch: region/first-layer `should_print`,
  `defer_unsupported = !is_infill_first` and `waits_for_infill` filtering.
  No prefix stub, unused support field or wipe-offset forwarding is added.

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
   nominal flow spacings from the aligned perimeter input. Keep prior
   sandwich and accepted thin/narrow behavior unchanged.
3. `perimeters/classic/chained_loops/types.rs::ExtrusionLoop` (and its
   constructors/copy carriers): add the upstream false-default field,
   setting it only at the source helper. The shared neutral loop type may
   carry this vocabulary; that does not implement Arachne's producer call.
4. Reuse `project_slice/island_print_order.rs`'s original region/collection
   ownership unchanged. Its existing regional emission adapter is
   `gcode_emit/motion/layer.rs`; there and in `motion/perimeter.rs`, implement
   the cited perimeter passes around existing infill. Preserve original
   collection/region ownership and complete region perimeter context for
   the existing wipe-before-external touching-lines gate. No printed-prefix
   support carrier is introduced in this nullopt branch. No detached second
   slicing pipeline or new public scheduling abstraction.

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
its named G-code pass consumer as one end-to-end source slice, specifically
its `wipe_support == nullopt` branch. The optional support object at
`GCode.cpp:8903-8907` is only emplaced when wipe is enabled, filament wipe
is enabled, `wipe_inward` is enabled, its distance is positive, and wipe
distance exceeds the source epsilon. Otherwise support preseed/append and
forwarding are no-ops. All three frozen RED inputs set `wipe_inward=0` and
remain valid for this exact branch. No prefix stub or unused field is added.

### Deferred: wipe_inward slice

The whole `wipe_inward` source boundary is separate and currently unported:
`WipePathHelpers.hpp::WipeInwardSupport`, `WipePathHelpers.cpp::append` and
the offset/support/material-side helpers; `GCode.cpp:8494/8738-8801/8847-8854`
forwarding and `offset_wipe_path_toward_support` execution; plus the optional
support preseed, append and forwarding at `:8903-8929`. Its typed option
projection and actual supported wipe geometry belong to that future slice,
not to this one. Existing `wipe_on_loops` is a different source behavior.

A read-only prerequisite audit stopped before any production edit or new
runtime on the approved RED. The explicit disposition is this nullopt-branch
boundary, not a waiver or a placeholder implementation. After this slice's
GREEN, propose a source-cited `wipe_inward` ARD as the next slice; its actual
RED and runtime budget require separate approval.

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

The initial bounded plan below was approved before fixture authorship.
Fresh qualification is recorded in the following section; no older
seam/overhang record is substituted for these actual runs.

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

The frozen RED budget was at most three cases, each Native2 fresh stable /
validated API1 / CLI1, with no retry, alternative producer, input search
loop or debugger probe. Exact inputs were frozen after the initial ARD
commit and before execution. If a Native premise or prerequisite fails
in any later authorized stage, preserve STOP rather than hunting geometry
or patching an adjacent boundary.

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

## Frozen Native-first public qualification

Exactly three input-only cases were frozen after the ARD commit and
before any Native execution. The deterministic stepped-solid generator
checks manifold orientation and volume; meshes/settings/member hashes
are retained in `tests/parity/classic-unsupported-wall-last/provenance.json`.

| Case | Input SHA256 | Native premise at physical Z1.4 |
| --- | --- | --- |
| Unattached ledge, enabled, three walls | `2520f0f1d96e28c610c1ed40435ca268da45a8eea5c401cd727c6b1262e0a611` | Infill precedes every closed wholly-overhang wall loop |
| Same ledge, disabled | `be3d9503721e1a2236064a7d80dacb6f90907b8a01acdbf5898ac159b84ef833` | Walls precede infill; complete strict output PASS |
| Attached eight-wall stack, enabled | `2aad0f555101c289b8cd1698bddda7bb520c3bbac58f51b11e30311dc411149c` | Anchored inner wall, unsupported tail innermost-out, then infill |

One frozen three-case run completed: Native6 fresh exports (three stable
pairs), validated API3 and CLI3 completed, raw API/CLI identical per case;
one strict PASS and two FAILs, zero ignored. Native premises are checked
before API/CLI, and complete shoulder extrusion passes/closed-loop
geometry were verified externally, not merely TYPE label presence.
No raw private loop flags or attachment-wave operands were observed.

The attached case first strictly fails at byte1712/line79: Native
`G1 X103.632 Y103.632 E.42247`, API/CLI `M73 P17 R3`. Its external
M73-command-only first difference is original716 on both streams:
Native travel `G1 X116.925 Y116.925 Z1.8` versus API/CLI
`G1 X119.775 Y119.775 Z1.8`, selecting the wrong first wall at the shoulder.
The unattached case first strictly fails at byte1291/line58:
`M73 P17 R3` versus `M73 P18 R3`. Externally its first non-M73 is
Native423/API422: `G1 X108.291 Y108.291 Z1.8` versus
`G1 X119.775 Y119.775 Z1.8`, selecting walls before the anchoring infill.
Full unfiltered contexts, maps and all removed M73 records are retained;
no Fword filter, coordinate/count golden or filtered acceptance.

The disabled control passes the unchanged complete comparator, including
M73, config/footer and trailing bytes. The two enabled cases are actual
strict REDs on the proposed producer/consumer boundary; no unported
prerequisite or API/CLI admission failure appeared in this frozen set.
Neither earlier M73 record is attributed as an independent timing root.
No production change, retry, extra probe, input variant, debugger, carrier
trace or family run occurred. Fmt and strict workspace/all-targets Clippy
pass. Only this result note changes after execution; no postcommit rerun
is claimed. Submit this frozen RED RR before any implementation.

## Literal nullopt port qualification

The existing region field macro projects the source false-default boolean
through process-region source, wire and resolved `RegionOptions`; no duplicate
registry definition is added. The Classic helper stably extracts the
unsupported tail, stably sorts descending inset, then uses original loop /
path / multipath polyline semantics and unsigned line-distance-tree queries
in separate attachment waves. It compares `sqrt(squared_distance)` strictly
against 1.5 times the maximum nominal scaled spacing. No geometry kernel,
threshold, formatter, topology classifier or sandwich body is changed.

`print_after_infill` defaults false in all constructors and survives clones.
Arachne's only mechanical change initializes this shared field false; its
producer remains deferred. Original collections are retained through the
three source perimeter passes, and only selected loops are copied for local
seam placement. First-layer/region `should_print` and already-infill-first
single emission are literal; no optional wipe support or unused prefix exists.

The scope was frozen before production edits. BEFORE ran on the immutable
amendment worktree; implementation and AFTER used a separate worktree and
fresh independent target. The exact same 298 test IDs ran once per phase:
137 PASS / 161 FAIL → 139 PASS / 159 FAIL, exactly the two enabled targets
F→P and zero P→F, zero ignored. There were 456 Native command receipts per
phase (453 code0, two238, one205); the existing Native failures are retained,
not relabeled as parity. The remaining159 family failures and FULL stay open.

Each phase includes fresh Native6 (three stable pairs), validated API3 and
CLI3 for the three frozen inputs, with raw API/CLI identity. AFTER all four
complete generator-only streams per case agree, retaining all M73 records,
footer, terminal whitespace and trailing bytes:

| Case | Bytes / lines | M73 retained | Complete normalized SHA256 |
| --- | --- | --- | --- |
| Attached stack | 67827 / 2594 | 89 | `5fa8595ffc4a1ca3e084ce93826245265484ef1a8db2627673bf2d5533a94528` |
| Disabled control | 56268 / 2141 | 85 | `e00d2b2e5d993c90246fd8370d91469e6179ca75d600161d47f9f93d925c3892` |
| Unattached ledge | 56370 / 2150 | 87 | `305888a79143847abfbc9ae6c71ee6c6ba17e27424c1689c9b1efb9a741b5486` |

Every extrusion run/point/line agrees over every physical layer; new full
and external non-M73 first differences are NONE for all three. No Native
private flag or attachment operand trace is claimed. Canonical generator
line is the sole strict normalization; diagnostic M73-only files are not
acceptance. No runtime retry, extra Native probe, input/producer/formula
variant, debugger, carrier trace, estimator overlay or test edit occurred.
Two static preflight command errors and two build/lint corrections are
retained: heredoc quoting, grep escaping, private-module import changed to
the existing re-export, and stable `sort_by` changed to stable Reverse-key
sorting. All happened before AFTER runtime; no suppression or kernel change.
Fmt and strict workspace/all-targets Clippy pass. This qualification note
is the sole postexecution tree change; no postcommit rerun is claimed.

## Approval and verification sequence

The Classic producer/flag/nullopt-consumer boundary and frozen RED are
approved. This doc-only boundary amendment must be committed on top of the
RED before any port. No implicit delegation or unrelated queue write is
authorized. Freeze Classic/wall-sequence/
overhang/region-infill/wipe before/after breadth before edits, require
zero prior-PASS to FAIL and report every flip and first remaining full/
non-M73 difference. Complete GREEN requires M73-inclusive whole streams;
COORD owns integration/FULL/closure. Preserve source/ELF/fixture/manifest
identities and every invalid attempt.

The approved narrow branch is not rebased or extended by this slice.
The initial ARD is based on the explicitly requested main5910. The literal
port adds only the approved metadata/Classic producer/nullopt consumer;
no independent pipeline design, root harness, runner, comparator, test or
fixture change is included. Propose the separate `wipe_inward` ARD next.
