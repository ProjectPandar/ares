# N-split Filled: bounded surface-writeback localization

Status: SPEC-REVIEW-REQUEST; localization complete, no implementation.
Base main57fea94989ff362129d0e53da0c35c298822f689 (FULL8 merged).
The docs-only checkpoint was rebased with range-diff1/1 `=`.
W3 odd-order was merged by coordinator; its original branch remains untouched.

The bounded window started2026-10-14 19:42:40 Asia/Singapore.
Coordinator paused it during FULL8,19:56:40–20:52:05 (3325s).
Adjusted deadline21:38:05; localization completed20:56:07.864,
1082.864 effective seconds (~18min). Runtime investigation is finished.
W5 clip/chain observation was explicitly CANCELLED and was not resumed.

## Exact upstream and Rust boundaries

PerimeterGenerator.cpp::process_no_bridge:1728–1929 owns this state change.
Classic process copies `Surfaces all_surfaces` at1212; process_arachne copies
it at2121 and invokes the shared function at2123. Surface.hpp:116 defines
`Surfaces` as std::vector<Surface>. The shared function retains a pointer at
1739, runs Filled nested-island intersection
and replacement at1826–1854, appends copied metadata and replacement polygons
at1841, erases the original at1845, repairs the current pointer only when
an earlier index is removed at1846–1848, and writes retained `last` through
the pointer at1922. Surface.hpp:38–43 defines the owning Surface fields;
ExPolygon/Polygon retain ordered contours and holes.

Rust destination for analysis is project_slice/perimeters/classic/
counterbore.rs::{process_surfaces,process_surface}, and
counterbore/filled.rs::{mask,recover_nested_islands}. RegionSurface adapts
the upstream Surface metadata and ExPolygon. This existing representation
is the compatibility shell being audited, not a new Ares admission or
wall-domain policy. Rust captures an original clone and later writes through
its current index at counterbore.rs:88 after nested processing.

Included: N-split Arachne Filled, layer1, one region, ordered input surfaces,
nested replacements, append/erase lifetime and retained current writeback.
Deferred: Classic M ownership/grouping, other counterbore modes, Arachne
wall support/materialization, bridge detector formulas, polygon normalization,
fitter/ETA changes, CLI profile-footer handling and oracle corrections.
This request proposes only the capacity/reference lifecycle described below;
no production, tests or comparator changes have been made.

## Frozen public RED and capture identity

W4 source evidence:
${LOCAL_EVIDENCE_ROOT}/worker4/counterbore-surface-6e034.md.
Frozen source root:/tmp/worker4-counterbore-bounded-o6df.
STL SHA256:
444935eac2493ed4f695e9fa635e8409fd4cad273ce817373a450c706bc9d807.
The input has48 vertices,92 facets and only two uses per indexed edge.
Neither geometry nor profile files are altered for debugger captures.
Machine/process/filament JSONs are byte-copied from the original public
orca-runner-Ql2Bub case. Counterbore mode is sacrificiallayer, Arachne.

W4 records both producers Native0, full-output DIVERGENT, first movement
change at layer1/local movement31, printZ0.52. Native has63 Overhang-wall
movements in that layer versus Ares0. Their layer0 movements agree. This
localization preserves the complete stream and all M73/movement assertions.

The existing owned Ares binary was built at7498165e0, with executable source
identical to approved cfff95d6a. Counterbore, filled, region-surface and
geometry sources match current main57fea94989 (audited by path-scoped
git diff against binary producer7498165e0, no changed files). The binary
is not claimed to have been built at the new main. Endpoint sorting is now
merged; the frozen STL has manifold edge-use counts. More directly, the
actual counterbore entry ordered polygon arrays match Native exactly.
That measured equality is the input prerequisite, not an assumption from
topology or a claim that different public adapters are interchangeable.

## Completed valid ordered-state observations

Seeded source-state root:/tmp/worker3-counterbore-seeded-state-npun_56f.
Actual function symbol start0x555556601620; entry stop skips20 prologue bytes.
The corrected stage hooks use symbol-relative instruction boundaries.
Source plus disassembly validate Surface96/ExPolygon56/Polygon32/Point16
byte strides, layer_id offset48 and slice_z offset56. No C++ DWARF layout
is claimed. Each serialized surface retains ordered contour/holes, surface
type, thickness, thickness_layers, bridge_angle and extra_perimeters.

| Native stage, layer1 | len/cap | Active base | Saved current pointer |
|---|---|---|---|
| Entry; current0, nested other1 |2/2|0x555558d89400|0x555558d89400|
| offset2 replacement ready,2 polygons |2/2|0x555558d89400|0x555558d89400|
| First append reallocates |3/4|0x555558df5470|0x555558d89400|
| Both appends, before erase |4/4|0x555558df5470|0x555558d89400|
| Erase other1 |3/4|0x555558df5470|0x555558d89400|
| Current0 retained-polygon writeback |3/4|0x555558df5470|0x555558d89400|

The saved pointer is outside the active Surface range after allocation moves.
Actual current0 writeback is to that saved address: the instruction path
assigns the contour vector at pointer+16 and holes at pointer+40. Active
ordered surfaces are identical before/after current0 writeback. Their current
ring keeps its10mm square hole half-width. Its prepared `last` has14.419999mm
hole half-width, with the same22mm outer contour half-width. Later current1/2
writes use addresses inside the active vector. Function return retains three
surfaces and the original current0 ring.

This Native GDB export exits0. The whole output matches W4's frozen Native
stream after only the EXISTING generated-by-line treatment. Raw capture SHA:
e7557e18bc373d4171b18a3b7bce849f031adee16c3a8d1f71aa7e9d080959aa;
frozen Native raw SHA:
66ecc9f1aa762bf86045d854cf7dfb71315a6dece0f4eda3e3eaa4ab6eac83ca.
The generator timestamps differ; no movement/M73/stat/footer is removed.

Fresh valid Rust root:/tmp/worker3-counterbore-ares-state-d8vwdaom.
Fresh official root:/tmp/worker3-counterbore-official-state-3vxnu7bo.
Both ran after FULL8 END on CPU12; GDB and inferiors exit0.
Native official also returns result0. No source/inferior values were changed,
no inferior functions called, no allocator manipulated. Existing instruction
breakpoints only observe active arrays, saved pointer values and live `last`;
freed Surface memory is not decoded.

| Comparison: Native seeded vs Ares | Result |
|---|---|
| Entry,2 surfaces/cap2 | Ordered polygons and all metadata equal |
| Nested replacement ready | Same ordered2 replacements; arrays equal |
| Before erase,4 surfaces/cap4 | Arrays equal |
| After erase,3 surfaces/cap4 | Arrays equal |
| Before current0 writeback | Arrays equal; ordered `last` equal |
| After current0 writeback | First captured live-array divergence, index0 only |
| Return | Other two polygons equal; index0 hole differs |

Spacing457079 and external width419999 agree. Metadata comparisons cover
surface type, thickness, thickness_layers, bridge_angle and extra_perimeters.
Native stInternal enum4 maps to Rust Internal by source, not by numeric
assumption. Ares current0 writeback equals Native prepared `last` exactly;
Native live array remains unchanged. This localizes the divergence before
wall generation, at the retained-polygon writeback, rather than a guessed
role or ETA correction. Optional optimized Rust indices are explicitly
unavailable at nested/pre-writeback stops; actual arrays/replacements/last
are readable, and post-writeback current indices0/1/2 are readable.

Official's actual entry/return ordered arrays and capacities agree with
seeded. Official intermediate instruction stages were not sampled; seeded
instruction offsets were never reused for that binary. Official raw stream
SHA256:b9e17ef15319e6432c9497de0e934782e148a9d1aa266611181c27be3fb597dd.
Both Native captured streams match the frozen Native stream with only the
existing generated-by-line treatment; no new normalization is proposed.

### Adapter boundary and complete-stream limitations

GDB used the existing Ares CLI load-settings entry, whereas W4's public RED
uses `ares_core::slice_with_profiles` with catalog/selection/overrides.
Their Ares output differs in exactly one line: the CLI `inherits_group`
footer contains three parent profile names, while the API and frozen Native
both emit `; inherits_group = ;;`. All other lines, including complete
movement, M73 and statistics streams, agree between those Ares outputs.
CLI raw SHA256:928fe5119f97ec1349d7709e92974f475d937221a8011a68ddf7937e00d71dca;
API raw SHA256:d27ab37712885e80ca54d23ae3cdc11a5d40086a5b1a63961b92bc43cb7f9f82.
Thus CLI/API complete byte equality is NOT claimed. The actual equal
counterbore input/state trace establishes core localization. The RED will
reuse the original public API adapter and retain its full footer comparison;
the unrelated CLI footer is deferred, never normalized away here.

### Corrections and VOID evidence

All roots are retained. Native1d_61z5q used an incorrect prologue-relative
stage mapping, so its stages are invalid. Rust zaezfcv2/lf_8p61x had
byte-pointer decoding errors. Rust p7b22wnz is a partial trace with missing
optimized locals; it is superseded by fresh d8vwdaom, not used to fill gaps.

Rust ha1m36hq is ENTIRELY FULL8 VOID: inbox START19:56:40 preceded its
19:57:06–19:57:14 execution. At the later process audit it had already ended;
killed PID list is empty. No kill/resume is falsely claimed and none of its
results count. Fresh d8vwdaom after END independently supersedes it.
The launcher now checks the latest exclusive START/END before any runtime
launch. No owned GDB/Native run remains pending.

## Approved deterministic rewrite slice; destination amendment

Coordinator's subsequent ruling permits reproducing the observed effective
Native outcome by deterministic capacity-growth rules, never by C++ UB.
The owning libslic3r boundary remains the Surface container and current
reference lifecycle above. Rust destination stays within counterbore.rs and
filled.rs; public input, load, selection and G-code contracts do not change.

Coordinator approved a779ad1d1 with this exact destination amendment,
which lands with RED before production:

- `crates/ares-core/src/project_slice/perimeters/classic/counterbore.rs::process_surfaces`
  owns the private logical Native capacity across traversal and the current
  reference-validity state captured/reset for each iteration. This is the
  Rust port of PerimeterGenerator.cpp::process_no_bridge:1739–1924; this
  function decides whether retained-polygon writeback updates the live slot.
- The same file's `process_surface` passes that state into
  `crates/ares-core/src/project_slice/perimeters/classic/counterbore/filled.rs::mask`
  and `recover_nested_islands`. The latter updates it on the ordered Surface
  append/erase operations (:1841/:1845–1848). It does not independently own
  or infer a second capacity from Rust Vec allocation.
- Final Surface appends (:1924) update the same traversal capacity in
  `counterbore.rs::process_surfaces`. No public state/config/API is added.

The accessible libstdc++15.3 headers describe the selected supplier rule:
bits/stl_vector.h:631–639 copy-constructs storage with source size;
:1417–1427 dispatches push_back to reallocation at capacity;
:2202–2209 computes `size + max(size, inserted_count)`;
bits/vector.tcc:558–566 _M_realloc_append requests that length for one item.
These are a source for an explicit compatibility operation, not a claim
that ELF/compiler comments identify the exact headers used for each TU.
Actual Native capture independently verifies copy2/cap2 and growth2→4.

Proposed minimum state is logical Native capacity and current-reference
validity, independent of Rust Vec capacity, pointer addresses or allocator:

1. At the copied Surface traversal, initialize logical capacity to count.
   Capture a valid logical current reference at each outer iteration.
2. For every Surface append, retain capacity when length<capacity; otherwise
   use the cited single-append growth rule and mark the current reference
   invalid. Preserve actual append order, polygons and metadata.
3. Preserve erase/index semantics. When an earlier surface is removed,
   source1846–1848 refreshes the reference: decrement current and make the
   logical reference valid again. An equal/later erase does not refresh it.
4. At retained current-polygon writeback1922, a valid reference updates its
   live indexed Surface. An invalid reference leaves the live Surface as it
   stands, reproducing the measured loss of the14.419999mm retained hole.
   Never dereference freed memory, inspect allocation addresses, arrange the
   allocator, insert unsafe code or add a fixture/coordinate/role predicate.
5. Preserve bridge-fill geometry, nested hole insertion and all metadata.
   Final emplace-back1924 participates in the same logical growth bookkeeping;
   no extra geometry policy or output normalization is introduced.

The frozen N-split trace has one retained current piece and two nested
replacement pieces. Other lifetimes requiring behavior not established by
source plus actual data are a STOP/amendment boundary, not permission to
invent a rejection, fallback or silently relax parity.

RED is N-split ONLY: reuse W4 public `slice_with_profiles` input/selection
path, commit the frozen input and generator/profile provenance as pure
inputs; Native official+seeded must accept/export,
and current Ares must retain the full-stream failure. Keep existing whole
comparison, movement/M73/stat/footer assertions and every unrelated test.
No new broad M matrix or test retirement. After SPEC approval and the
separate RED commit/evidence report, an independent small GREEN commit must match full output in both modes,
preserve existing counterbore controls and pass strict/fmt/wasm before review.
Workspace/full remain explicit coordinator allocations.

Local source-header path:
/nix/store/qrn789mrv652j7f5bcjp367npzpcyvdi-gcc-15.3.0/include/c++/15.3.0/bits/.
The compatibility rule is included here for reviewer selection; it is not
yet implemented or treated as universally guaranteed Native behavior.

## Durable evidence receipt

Archive (outside Git; includes valid, corrected and VOID roots, commands,
full ordered state JSONL, source assembly, selected supplier headers, original
public inputs/outputs, adapter-difference receipt and timing/agreement JSON):
${EVIDENCE_ROOT}/worker3/counterbore-n-split-localization-20261014.tar.gz
Bytes:380044; SHA256:3909c4bcebdea683f23a5cf1e996165b3c4b190c5dc5e8bda75d1a2abf826397.
Cache directories and target binaries are excluded. No Native products enter
Git. The committed stage table, input hash, stream hashes and correction
ledger above preserve the conclusion if /tmp is cleaned after10days.

This docs-only request ran no new builds/tests/full/workspace. After approval,
all directed outer processes remain taskset CPU12 with fresh cache/artifacts,
CARGO_BUILD_JOBS=1 and assigned window; full/workspace need explicit allocation.
No push or production change is authorized by this SPEC request.
