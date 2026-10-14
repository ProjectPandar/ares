# N-split Filled: bounded surface-writeback localization

Status: draft during FULL8 freeze; final SPEC request follows END. No
production or test change authorized.
Base main6e0349d9f6da4c0dfdae3cedb26ef9cc0a98e412. W3's approved
odd-order candidate cfff95d6a remains untouched on its separate branch.
Coordinator reassigned this analysis ahead of first-layer follow-up.
Window:2026-10-14 19:42:40–20:42:40 Asia/Singapore,3600 wall-clock seconds.
FULL8 freezes runtime work; this deadline is not silently extended.

## Exact upstream and Rust boundaries

PerimeterGenerator.cpp::process_no_bridge:1728–1929 owns this state change.
Classic process copies `Surfaces all_surfaces` at1212; process_arachne copies
it at2121 and invokes the shared function at2123. Surface.hpp defines
`Surfaces` as std::vector<Surface>. The shared function retains a pointer at
1739, runs Filled nested-island intersection
and replacement at1826–1854, appends copied metadata and replacement polygons
at1841, erases the original at1845, repairs the current pointer only when
an earlier index is removed at1846–1848, and writes retained `last` through
the pointer at1922. Surface.hpp:37–44 defines the owning Surface fields;
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
fitter/ETA changes, and any production/oracle correction. No guessed mask,
index, actual invalid-reference emulation or output-comparator change is
included without a separate reviewed implementation slice.

## Frozen public RED and capture identity

W4 source evidence:
/home/indexyz/.ares-evidence/worker4/counterbore-surface-6e034.md.
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
geometry sources match main6e0349. Its only production delta is endpoint
sorting; the frozen STL has manifold edge-use counts. More directly, the
actual counterbore entry ordered polygon arrays match Native exactly.
That measured equality is the input prerequisite, not an assumption from
topology or a claim that different public adapters are interchangeable.

## Completed valid observations before FULL8

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

Valid partial Rust root:/tmp/worker3-counterbore-ares-state-p7b22wnz.
Entry has identical two ordered polygon arrays; spacing457079 and external
width419999 match Native. Its readable current0 post-writeback ring equals
Native's prepared `last` exactly: hole half-width14.419999mm. Final ordered
arrays have three surfaces; only polygon index0 differs from Native.
Both later polygons agree exactly. Source collection divergence therefore
exists before Arachne wall generation. This is stronger than a role/ETA
observation, but not yet a complete earliest-intermediate-state trace.

## Missing fields, freeze and exit criteria

Rust nested/current/pre-writeback locals were optimized out in the valid
partial attempt. Those errors are preserved, not filled with inferred data.
A corrected helper preserves readable ordered surfaces even when optional
local-index fields are unavailable; its latest runtime root
/tmp/worker3-counterbore-ares-state-ha1m36hq is ENTIRELY VOID: FULL8 started
in inbox19:56:40, this leg ran19:57:06–19:57:14. By the freeze process audit
it had ended; killed PID list is empty. No kill/resume is falsely claimed.
It contributes no localization evidence. No active owned runtime remains.

Also retained: first Native root1d_61z5q has incorrect prologue-relative stage
mapping, and Rust roots zaezfcv2/lf_8p61x have byte-pointer decoding failures.
They are not successful stage captures. All corrections remain explicit.

Only after FULL8 END, and only before the original deadline, complete:

1. Fresh Rust nested replacement/before-and-after erase/pre-and-post current
   writeback arrays. Preserve unavailable-field annotations, not guessed indices.
2. Official actual function entry/return and corresponding verified instruction
   stages if available within the cap. Seeded instruction offsets cannot be
   assumed valid in a different producer.
3. Compare ordered arrays stage by stage. Report the earliest measured mismatch
   and distinguish it from source-derived conclusions. Check each captured
   producer's full stream against the frozen stream without extra normalization.

STOP at deadline or missing trustworthy state; publish the exact remaining
uncertainty. Existing observations establish Native relocation plus a saved
non-current address and later active-domain divergence. They do not authorize
Rust to reproduce an invalid C++ lifetime, modify Native, repair downstream
roles, weaken the public RED, or claim a safe geometry/index fix exists.
A production slice requires SPEC approval after localization. No production
changes, new tests, full/workspace or push here.

## Proposed deterministic rewrite slice for approval

Coordinator's subsequent ruling permits reproducing the observed effective
Native outcome by deterministic capacity-growth rules, never by C++ UB.
The owning libslic3r boundary remains the Surface container and current
reference lifecycle above. Rust destination stays within counterbore.rs and
filled.rs; public input, load, selection and G-code contracts do not change.

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

RED is N-split ONLY: commit the frozen input and its generator/profile
provenance as pure inputs; public Native official+seeded must accept/export,
and current Ares must retain the full-stream failure. Keep existing whole
comparison, movement/M73/stat/footer assertions and every unrelated test.
No new broad M matrix or test retirement. After RED is reported and approved,
an independent small GREEN commit must match full output in both modes,
preserve existing counterbore controls and pass strict/fmt/wasm before review.
Workspace/full remain explicit coordinator allocations.

Local source-header path:
/nix/store/qrn789mrv652j7f5bcjp367npzpcyvdi-gcc-15.3.0/include/c++/15.3.0/bits/.
The compatibility rule is included here for reviewer selection; it is not
yet implemented or treated as universally guaranteed Native behavior.
