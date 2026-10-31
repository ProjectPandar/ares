# ARD: beta first-layer Brim speed selection

Status: source boundary and public seam approved; RED before production code.
Later batch only, not frozen B13. Authority: OrcaSlicer v2.5.0-beta,
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Owning upstream boundary and Rust destination

Port only `GCode.cpp:9458–9464::_extrude`: on the first layer **or** the first
object layer over raft, excluding `erBottomSurface`, choose
`initial_layer_speed` for `is_perimeter(role) || role == erBrim`; choose
`initial_layer_infill_speed` for other roles. `ExtrusionEntity.hpp:62–67`
defines `is_perimeter` as internal/external/overhang perimeter, not Brim.

Rust destination: the first-layer-like branch of
`project_slice/gcode_emit/motion/features.rs::PathProperties::speed`.
The existing adapter implements the source role/layer speed calculation but
omits Brim from this particular predicate. Replace that missing leaf, not the
role vocabulary or the motion pipeline.

**Non-negotiable:** the adjacent `slow_down_layers` branches at
`GCode.cpp:9466–9489` use `is_perimeter` **only**. Keep the Rust `is_perimeter`
classification and `slow_down_layer_ramp` calls unchanged; do not globally
classify Brim as a perimeter. Preserve BottomSurface exclusion, positive skirt
speed override, requested/base speeds, acceleration/jerk, actual flow caps,
over-raft guard and existing role identities.

## Read-only localization and selected shared cause

The three original shortest-edge offset fixtures are qualified strict public
REDs. They share the first M73 mismatch at line32. Both Native and API export
the same distinct first-layer and first-layer-infill speeds. Native Brim feeds
use the former; API feeds use the latter, and the wipe inherits that feed.

Across each complete executable block, read-only alignment identifies26 feed
changes and no other non-progress command/XY/E differences. The Brim geometry
and extrusion amounts match. This alignment is diagnostic attribution only,
not a modified acceptance comparator or a claim of public GREEN.

The initial M73 mismatch plausibly follows from the total-time denominator
changed by Brim feeds, despite occurring before Brim emission. That causal
claim must be tested through the complete public stream region. Do not adjust
M73 counters, planner times, rounding, coordinates or tolerances to obtain it.
If M73 persists after the literal speed predicate port, keep it OPEN and cite
the remaining source owner; no counter fudge or progress masking.

The nested/other completed motion prefixes reach the already separate
configuration/footer/statistics boundary. Do not grow this leaf into their
configuration rewrite or reinterpret those original full-stream REDs as GREEN.

## Agreed public RED and GREEN boundary

Use the same three immutable inputs from `tests/parity/clipper-wrapper-geometry`:

| Input | SHA256 |
| --- | --- |
| round-offset-short-edge-below-threshold.3mf | ce8aa1e13b8f867acad2743d33cdec3a667018613066e6701ec0622f00eeeb97 |
| round-offset-short-edge-at-threshold.3mf | 99a0afc16380d62a2da6dd956a338ecc23a592b5586297274cb7e17f8907ec9a |
| round-offset-short-edge-above-threshold.3mf | 6b53a7623b9dfca2271c8d138aca5f2b667693d145f7da648f15836f3902fe24 |

The public seam is `slice_project_plate_validated` byte output versus the
qualified official beta Native producer. Select the **complete executable
block** between its explicit START and END markers at ordinal0 in Native,
and compare API ordinal0, without searching for a favorable API feature.
Include the markers and **every** record: custom preamble, initial travel, all
M73, every feed/XY/E move, wipe, fan and end G-code. Config/footer outside this
source-declared block is not part of this new scoped comparator; the original
seven strict full-stream tests and parity ROOT remain completely unchanged.

Qualify the input hash, Native Brim activation, stable official double replay,
and complete pre-fix API output before production work. Persist the selected
blocks and ordinal metadata externally. No Native bytes, expected fragments,
goldens, fixture branches or new fixtures are committed.

1. Write this ARD before new tests or production changes.
2. Add three tests in the existing native-stream grouping, qualify runtime RED
   on the unchanged speed code, then freeze a separate RED commit.
3. Add Brim only to the first-layer-like speed predicate. No other production
   change; prove exact executable-block equality on all three inputs. If progress
   remains RED, stop and retain OPEN rather than narrowing the comparator.
4. Run the original owned directed cases, fmt and strict workspace/all-target
   Clippy. Preserve both island and both width scoped GREENs. Report the seven
   full-stream residuals honestly; no FULL, seeded sweep or ledger rerun.

## Runtime RED qualification before the port

All three complete executable-block comparisons are runtime RED on unchanged
production code. Official Native double replay is stable; both streams expose
exactly one executable block and compare ordinal0 against0 with both markers
and all records. Native/API block lengths are41969/41947 (below),41994/41972
(at),41989/41967 (above). The first mismatch is the progress record at byte429,
blockline16 in every case. All three public APIs admit the immutable inputs;
complete pre-fix API streams are retained externally. Fmt and strict workspace/
all-target Clippy pass, with no introduced suppressions.

Included: the first-layer and over-raft speed-selection leaf, with inherited
wipe/progress effects observed through the existing consumers. No separate
planner/progress implementation is added. Over-raft uses the same source branch
but is source-audited rather than directly activated by these three inputs.
Deferred: unrelated first-layer plane/belt support, slowdown-layer ramp rewrite,
new configuration modes, geometry/Clipper changes, footer/config/statistics and
any independently exposed progress residual. No additional instrumentation.

All files remain below400 LOC; helper arguments<=5, no new suppression in any
intermediate commit. Keep the required coauthor trailer and later-batch queue.
