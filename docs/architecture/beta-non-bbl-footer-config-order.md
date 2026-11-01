# ARD: non-BBL config-before-statistics footer order

Status: seven complete post-config tails are runtime GREEN after the literal
ordering port. Original seven full streams remain OPEN inside config schema.
Later batch. Authority: OrcaSlicer v2.5.0-beta,
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Owning upstream boundary and Rust destination

`GCode.cpp:4882–4935::_do_export` writes the non-BBL tail in this order:

1. CONFIG block, only for non-BBL printers and when config output is not skipped.
2. `DoExport::update_print_stats_and_format_filament_stats` output.
3. Non-BBL compatible totals/layer/time statistics, followed by the source
   trailing newlines.

BBL writes its config in the header (`GCode.cpp:4285–4300`), not this tail.
This is an upstream export-layout port, not a new Ares footer pipeline.

Rust destination: `project_slice/gcode_emit/orchestration.rs` tail calls and
`gcode_emit/finish.rs::append_compatible_stats`. The current shell appends
filament statistics, then compatible statistics, then the cached config block.
Relocate only the existing config append to the source position before both
statistics calls. Preserve the non-BBL guard and the existing optional cached
config (`None` still means no block). Keep the BBL header append exactly where
it is and do not append a second BBL config at the tail.

No statistic values, processor accounting, config payload, field membership,
legacy rules, serialization, geometry, motion or M73 calculation changes.
The surrounding first/end G-code, completion controls, EXECUTABLE_BLOCK_END and
final trailing-newline phase remain unchanged. Literal source phase ordering
must produce correct whitespace; do not splice or replace footer strings after
export, move serialized lines by content, or normalize the comparator.

## Read-only evidence and independent config-schema owner

The same original seven immutable geometry inputs all first differ at this
layout boundary. All nine nonblank statistic records already equal Native,
including amounts, material costs and both estimated times. Native's complete
post-CONFIG_END tail is305 bytes; API's tail is only two newline bytes because
its statistics precede config. Native's tail equals the existing misplaced API
statistics plus the source final newline. This is diagnostic localization,
not permission to create a whitespace filter or to hardcode output.

There is a separate, uniform config-schema gap: Native740 records versus API625,
116 missing Native keys, obsolete API `silent_mode`, and `enable_arc_fitting`
serialized as Boolean rather than the beta enum. Other common ordered config
records match. The old `orca_block_keys` membership explicitly cites2.4.2.
Those definitions/keyset/legacy/type ports are routed to the **config owner**,
not included in this ordering leaf. In particular do not fabricate default
config lines, map the arc value in a footer string, revive obsolete keys or
change the payload cache as a workaround. The original seven full-stream REDs
remain OPEN for that owner even after scoped statistics-tail GREEN.

The new `gcode_skip_config_block` option's schema/transport belongs to the same
config owner; this slice preserves the adapter's existing skip/optional-block
behavior rather than introducing an independent option or fallback.

## Agreed test-first public seam

Reuse all seven original inputs under `tests/parity/clipper-wrapper-geometry`,
with their existing immutable SHA256 identities and no packaging/config edits:

- nested-hole-island.3mf
- holes-touch-after-compensation.3mf
- collinear-boundary-with-thin-spike.3mf
- coincident-width-overhang-intersections.3mf
- round-offset-short-edge-below-threshold.3mf
- round-offset-short-edge-at-threshold.3mf
- round-offset-short-edge-above-threshold.3mf

Use public `slice_project_plate_validated` bytes against qualified official beta
Native output. The Native config must follow EXECUTABLE_BLOCK_END, activating
this non-BBL branch. Select the complete bytes **after CONFIG_BLOCK_END marker
ordinal0 through EOF**, and select API ordinal0 through EOF in the same way.
Include every statistic and **all leading/trailing blank lines**, with no
trimming, record filtering, line reordering or convenient API feature search.
Both streams must expose exactly one config-end boundary. Native replay must be
stable, all input hashes qualified, and complete pre-fix API bytes captured.

No Native stream, golden or expected output fragment is committed. Marker names
are source boundary vocabulary, not frozen expected data. Configuration payload
is outside this new source-declared post-config seam; all original seven strict
full-stream comparisons and parity ROOT remain unchanged and OPEN.

1. Write this ARD before the new tests or production edits.
2. Add seven scoped E2E tests in the existing native-stream grouping, qualify
   the305-byte-versus2-byte runtime REDs on unchanged production code, and freeze
   a separate RED commit.
3. Move the literal cached-config append before both statistics phases, with
   the same non-BBL/optional guard. Prove all seven complete tails byte-equal to
   unchanged Native; no string hacks or statistic-value edits.
4. Run owned directed gates, fmt and strict workspace/all-target Clippy. Retain
   all existing island/width/executable-block GREENs. Report where the original
   seven full-stream cases next differ inside config; do not relabel them GREEN.

## Runtime RED qualification before the order port

All seven public cases are runtime RED on unchanged production code: Native
post-config tail305 bytes versus API2 bytes. Both streams expose exactly one
CONFIG_END marker and compare ordinal0 through EOF with every byte included.
Native config follows executable output, so the non-BBL branch is active.
Official double replays are stable; all public APIs admit; complete pre-fix API
streams are retained externally and match the accepted checkpoint hashes.
Fmt and strict workspace/all-target Clippy pass with no new suppressions.

## Literal order port and directed result

The separate RED is frozen in `9ff58ee03`. Moving only the cached config append
before both statistics phases makes all seven complete post-config tails305
bytes, byte-equal to the unchanged runtime Native tails. Leading/trailing blank
lines are included without trimming. Native double replays remain stable.

Before/after complete API captures prove that every header/executable prefix
and every config payload is byte-unchanged, and total output lengths are
unchanged. Only source phase order changes; no statistic value or serialization
calculation is rewritten. BBL's existing header append remains untouched, and
the relocated tail append retains both non-BBL and optional-block guards.

The final bounded owned21 run has14 scoped PASSes and seven retained strict
full-stream REDs. All original APIs admit, CLI exits0, rawAPI==CLI and official
replays are stable. Each strict case advances eight lines into config, first
missing the same beta adaptive-infill key; the keyset/type/legacy work remains
with the config owner. Existing island/width/executable-block GREENs are retained.
Fmt and strict workspace/all-target Clippy pass; no suppression, new fixture,
probe, FULL, seeded sweep or ledger reclassification is introduced.

BBL/header and optional-block skip behavior are source-audited preservation
boundaries; these seven inputs activate the non-BBL emitted-config branch, not
direct BBL or config-skipped coverage. No additional fixture/probe is included.
No FULL, seeded sweep or beta-ledger classification rerun. No extra crate or
platform I/O: production stays in the platform-neutral emission adapter.

Files stay below400LOC, helper arguments<=5, no new #[allow]/#[expect] in any
intermediate commit; keep the required coauthor trailer and later-batch queue.
