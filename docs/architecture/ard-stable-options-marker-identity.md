# Stable OptionsZCorrector marker identity

## Source boundary

Upstream pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This slice ports the approved correction in
`tools/chain-probe/procgcode-stable-options.patch:8-30`, around
`GCodeProcessor.cpp:5951-6008` actual-speed insertion and
`GCodeProcessor.hpp:688-705` pending-marker relocation.

`docs/architecture/web-colorchange.md:128-189` approves stable pending-marker
identity for viewer moves rather than the upstream stale-index defect.
`web-marker-time.md:119-129` retains unmodified official G-code/M73 authority.
The correction is not a nondeterminism workaround: three raw Native runs of
the tagged input produce the same 7348 vertices, including phantom1653.

## Rust destination and non-negotiable decisions

The existing `processor/estimate/options_z.rs::Log`/`Replay` represents the
upstream interleaved parser/planner move lifecycle. Extend that mirror, not
the vgcode converter or WASM adapter. Its schedule remains a temporary
compatibility representation of upstream `calculate_time`, not an Ares-owned
time estimator.

1. Remap the pending marker immediately after each actual-speed insertion,
   using the last ID-map entry whose old index is at most the pending index.
   Add that entry's `new - old` delta. This is the patch's `upper_bound` then
   predecessor operation, not exact-match lookup or total-insertion offset.
2. Public `GCodeProcessorResult::moves`, including CLI `--dump-moves`, uses
   stable marker identity. Do not add a selectable raw-move API or a second
   stored public viewer view.
3. Export-cache eligibility still follows the unmodified upstream replay:
   stale relocation can change which queued block is cacheable. Preserve
   that raw replay for official M73, and replay the same recorded moves and
   already-computed schedule with stable identity for public moves. Share the
   replay implementation; do not recompute the schedule, planner arithmetic,
   statistics, stop times, or template totals.
4. Marker-free input retains one replay. Only input with marker calls needs
   the second move-list replay; neither list survives as a duplicate public
   result. These are required export/viewer authority boundaries, not a
   fallback, compatibility feature flag, or user configuration.
5. Keep raw Native captures. Tagged controls must prove the complete
   raw-to-Rust XYZ-bit difference equals raw-to-stable Native, including
   missing tails, and compare move identities against stable Native.
   Untagged controls, metadata and statistics remain raw-official. No fixture
   changes, exclusions, looser tolerances, or raw-for-stable substitution.

## Included and deferred behavior

Included: pending ColorChange/PausePrint/Custom_GCode marker indices through
actual-speed insertions and their resulting public processor/vgcode vertices.
The ordinary copy/erase update, path-start phantom rule, and timing arithmetic
remain their existing upstream ports.

Deferred: other processor corrections, planner/M73 redesign, marker matching
by G-code ID, GL viewer runtime, and broad processor cleanup. The raw move
authority described in `gcode-processor-options-z-moves.md` remains applicable
to export-cache membership only; its public-move authority is superseded by
the approved stable-marker exception above.

## Existing RED and verification

The authorized RED is
`ares-wasm::marker_time_tagged::tagged_normal_time_matches_every_stable_native_vertex`:
on the rebased MK4 tip it fails at line70, Rust7348 versus stable Native7347,
with input SHA256
`55ec96a28d3ac035d94d0923c5f3be53cb9d897963fc2b8a561b7291ef935f89`.
Run it with `ARES_REQUIRE_ORCA=1` and both live marker-time producers.
Restore contradictory raw-pinned tagged controls to the approved stable
authority before implementation, retaining their raw captures and exact
bounded-difference assertions.

GREEN must pass every tagged stable ID, both duration modes, and ordered-f32
Normal elapsed bits, plus the Native raw/stable bounded-impact proof. Run
full official and seeded G-code parity, full non-parity with required live
oracles, fmt and strict clippy. All own work stays CPU13/thread1/build-jobs1.
Report any baseline failures by exact binary/test name; do not claim an
all-green workspace when baseline failures remain.
