# Nearest-seam embedded-distance layer gate — source rewrite SPEC

Status: bounded read-only causality confirmed; docs then tests-only public RED.
Base `4af042def` (seam visibility mode gating already merged at that
commit).
Post-merge re-verification done: `alignment::prepare` keeps its `layers` argument
(that base commit only widened `visibility` to `Option<&GlobalVisibility>`), the wrong
predicate still sits at `preparation.rs` `embedding_layers`, and
`OrderedExtrusionIsland.perimeter_regions` remains populated at seam-prepare
time (`island_print_order.rs:26,164,188`) and is only later taken by
`gcode_emit/layers/tool_buckets/object.rs:26`. NO production edit or GREEN
authorized here.

## Upstream boundary and mandatory semantics

Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `GCode/SeamPlacer.cpp:1071-1077`: while visiting layer regions, SeamPlacer
  counts `regions_with_perimeter` by `region->perimeters.entities.size() > 0`;
  only layers where that count exceeds one join the embedding pass.
- `GCode/SeamPlacer.cpp:1095-1097`: on counted layers
  `populate_embedded_distance` computes `distance + 0.65f * flow_width` per
  candidate (hidden interior candidates score negative).
- `GCode/SeamPlacer.cpp:770-775`: `SeamComparator::is_first_better` applies the
  hidden embedded gate (embedded < -0.5) ahead of the remaining terms.
- `GCode/SeamPlacer.cpp:1461,929-940,1554-1556`: every seam mode including
  nearest runs `pick_nearest`; `1527-1560` perimeter walk;
  `1564-1627` projection/depth/split.
- `src/libslic3r/LayerRegion.cpp:82-84,119-122`: `m_perimeters` is filled only
  by the wall-generator append points.
- `PerimeterGenerator.cpp:1568-1569` classic appends only non-empty perimeter
  collections; `:229-232` thin walls join the collection;
  `:1613-1621` gap fill never enters perimeters;
  `:2128,2471-2476` arachne append points.

Exact Rust destination boundary is
`crates/ares-core/src/project_slice/seam_placement/alignment/preparation.rs::embedding_layers`
(layer predicate ONLY). The embedded-distance formula
(`alignment/context.rs:118-127 populate_embedded_distance`), the nearest walk /
comparator / split (`runtime/nearest.rs place_nearest_penalized`) are already
literal ports and stay unchanged. The current compatibility shell counts
`any(|surface| !surface.roots.is_empty())` from fill-surface seeds, which is
not the upstream count of appended perimeter collections: on the witness
layer 45 both regions carry empty fill-surface roots, the gate reads false,
`populate_embedded_distance` is skipped and every candidate keeps embedded 0.

The port slice restores the upstream layer predicate over ordered-layer
islands: layer joins the embedding pass iff
`islands.iter().flat_map(|isl| isl.perimeter_regions.iter().copied())
.collect::<BTreeSet<_>>().len() > 1`. This is the coordinator-ACKed main plan
(rev-2/rev-3 of `seam-nearest-multipath-loop-proposal.md`; record-surface
forms were disproven or withdrawn there — including the round-1
`!record.surfaces.is_empty()` predicate, which entity_collections' 1:1 surface
seeding would make always-true). `perimeter_regions` is fed by
`perimeters/classic/perimeter_append.rs:339-347 append_nonempty`, the literal
port of the upstream append boundary, so per-region collection non-emptiness
matches `entities.size() > 0` including thin-wall-only regions and excluding
gap fill. Included: this predicate swap only. Deferred: everything else
(formula, comparator, walk, split, alignment, the visibility mode gating
landed at 4af042def).

No writer/time/M73, travel/origin/lift, processor/fan/tokenizer, brim, seam
config, formatter or geometry changes. No fallback, feature flag or hardcoded
Native seam index.

## Frozen real Native witness

Original thin-gap-j1 Snapmaker J1 IDEX stock inputs, not regenerated:

- `wall-arachne.3mf` SHA256
  `5f822b0761a325deb15795f82dedb9a7d03a8c40304ed389edaf4e4543b351c7`
  (evidence root `/home/indexyz/ares-evidence/w3-geometry-discovery3-78d642702/thin-gap-j1/inputs/`).
- Dual-mode sibling `wall-classic.3mf` SHA256
  `482eb2e6a19b43a56cc13553dc9c8795ca9d4ccf9510743d45d236b547cab01d`:
  sole effective delta `wall_generator arachne->classic` inside
  `Metadata/project_settings.config`, every other member byte-copied, rebuilt
  zip with fixed 1980 timestamps. Native `--slice 0` exit 0, T0=40/T1=39,
  11367 lines.

Causal record (evidence `/home/indexyz/ares-evidence/w3-arachne-thinwall-5e8be001f/`,
`w3-thinwall-stop-seam-handoff.md`, proposal rev 3): first divergence record
60/61 at layer 45 (print_z 9.2 mm), entity e0 = object0/region0/entity0, a
51-path widened inner loop `erPerimeter` under seam `nearest`. Structural
layer content, island assignment, emission order and tool order are
bit-identical; the 125-width chains are equal as multisets. Both engines hold
145 identical candidates, walk range (88,108), cursor (1.272,-2.335). Upstream
cand[88] embedded -1.1059 (hidden) wins; Ares best=94 because every embedded
value is 0 (gate false). Upstream seam vertex local (-5.000,-8.934) world
(111.000,106.176) vs Ares (-1.851,-2.181) world (114.149,112.929). Earlier
candidate intervals match because their ranges hold no hidden point (the first
hidden-point interval is [88,108)).

Public-RED diagnostics (saved artifacts
`/home/indexyz/ares-evidence/w3-arachne-thinwall-5e8be001f/parity-artifacts/thin-gap-j1-seam-embedding-wall-arachne-SFXtn8/`):
strict first difference at byte 2150 line 150 is an M73/G0 ordering flip —
early propagation, not repaired and not ignored (same early-propagation
shape as the frozen rear-seam visibility pair at 89a18865e; the
dual-mode control passing byte-equal on the same printer/M73 logic rules
out a general M73 residual). First non-M73 physical difference: line 5833,
`G1 Z9.2` (layer 45, print_z 9.2 mm), `F2472` vs `F2474` followed by the
reordered seam-coordinate chain — the embedding-gate divergence itself,
same layer as the causal record. Raw API and CLI outputs are byte-equal
cmp-verified because the canonical assertion panics before the final
helper update on RED.

## Public RED and approval gates

Tests-only RED after this docs commit: two source-cited public cases
(`wall-arachne`, `wall-classic`) via the UNCHANGED
`native_stream_case::check` -> `NativeCase`, fixture SHA freeze,
`(true, |_,_| {})` both-tools assertion. `wall-arachne` is the RED target and
must be an actual strict FAIL, not a caught expected error; `wall-classic`
runs the same witness in the other generator mode (RED if it also strictly
fails, otherwise it stays as the dual-mode positive control). Full ordered
comparison retains M73/time/motion/comments/metadata/footer; no masks,
tolerances, frozen reference text or expected seam index in public tests.

Run gates: CPU12 taskset, jobs 1, threads 1, CARGO_INCREMENTAL=0. Official
Native sliced twice FIRST (replay determinism enforced by the unchanged
harness), then validated byte API and real CLI. Existing orca_parity
neighbors (including the rear-seam visibility pair) must stay PASS. Because the canonical
target assertion panics before the final raw helper update, independently
confirm the saved API/CLI raw equality; never claim later assertions ran on
RED.

Report docs SPEC + tests-only RED with exact tips, Native2/public receipts,
clean scope; HOLD. No production edit, GREEN, merge/push or integration
implied. Implementation requires explicit separate approval. If a different
unexpected RED residual appears, STOP and report without repair.
