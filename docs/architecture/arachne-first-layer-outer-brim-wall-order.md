# Arachne first-layer outer-brim wall ordering

Status: proposed. Selected source-cited rewrite and public witness. The first bounded
Native qualification is stable and observes exterior-first walls; public API
sensitivity is required before a public RED. Production is unchanged.

## Upstream boundary

OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `PerimeterGenerator.cpp:2909-2911::process_arachne` adds
  `outer_walls_first_for_brim()` to first-layer wall-order eligibility.
- `:3204-3211` defines physical layer zero, positive brim width, and brim types
  OuterOnly, OuterAndInner or LeadingEdgeOnly as the upstream predicate.
- `:581-584::traverse_extrusions` uses the same predicate to suppress thin-wall-hole
  collection reversal when the brim requires the exterior first.

Old `8500fcdccaa10b5099ac20d252af3a7c560046f1` at `:2273-2280` chooses first-layer
order only from wall_sequence. This is a beta boolean eligibility rewrite, not a
new Ares wall-order feature or a geometry compensation.

## Rust destinations and included boundary

All destinations are under `crates/ares-core/src/project_slice/perimeters/arachne/`:

- `config.rs::validate_record/is_outer_wall_first` currently ignores object brim
  type/width when deriving the existing outer_wall_first flag.
- `materialize.rs` passes the raw region wall sequence into traversal.
- `traverse.rs` suppresses thin-hole reversal only for explicit OuterInner; it lacks
  the additional first-layer brim condition.

The proposed slice covers both Arachne consumers of the same eligibility rule,
using existing accepted OuterOnly/OuterAndInner kinds and positive width at layer
zero. Existing types AutoBrim, BrimEars, Painted, InnerOnly and NoBrim do not qualify.
LeadingEdgeOnly is not represented by current ProcessBrimType: its import/execution
is deferred, not invented or silently admitted in this slice.

Classic's already-present OuterOnly override and its missing OuterAndInner extension
are an adjacent generator boundary, not covered by an Arachne-only witness. Keep
raw wall constraints, beading, line geometry, C2 arithmetic and sandwich algorithms
unchanged. No new crate, configuration option or legacy fallback is required.

## Why inset transfer is not the first independent slice

`PerimeterGenerator.cpp:565-566` assigns closed Arachne loop inset identity;
Rust `traverse.rs:199-201` still stores -1. However Arachne sandwich ordering runs
before extrusion materialization and reads ExtrusionLine.inset_index, not that
field. Current observable downstream use is chiefly the all-overhang seam
extraction gate, which encounters the separately open C2 numeric dependency.
Do not assert an independent wall-order RED or add a private metadata-count test.
Open MultiPath transfer at `:595` is also outside the closed-loop prerequisite.

## Proposed public qualification

The witness is the ordinary two-wall Arachne prism, inner/outer wall
sequence, changed only to OuterOnly brim width 3 and disabled one-wall reduction.
Input SHA-256: `c844cc32b42f635ed24e494c01492c70727d28c7bcbade92b19a7951512f3f1c`.
The first physical layer has no preceding-layer overhang score, so the known
positive-overhang rounding defect is not the proposed deciding mechanism.

The first Native-only qualification has stable double replay, emitted brim before
the first object, and a 343-byte complete first-layer object block containing
Outer wall followed by Inner wall. No Native output is committed.

Qualification is bounded to two distinct official-beta Native inputs. Public
qualification must show successful validated slicing with the opposite wall order,
then retain strict runtime Native/API equality of the complete original first-layer
object block. The same boundary already used by shell-gate witnesses naturally
excludes the preceding brim, but omits nothing within the object. A fixture with
no inner wall, an unobserved brim override, a different seam/C2 numeric gap or
unsupported brim import is not a production RED.

The Native-only qualification precedes repository test/fixture edits; production
must remain unchanged until a public RED exists. No angle/string/normal change
or comparator relaxation may be absorbed. Combined C2 verification
of the previous Nearest witness is external evidence, not this candidate's budget.
