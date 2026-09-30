# Internal-solid variable-width implementation plan

1. Preserve the original strict KSR replay RED as the feedback loop; confirm
   repeatability and minimize the scenario one variable at a time.
2. Compare native/Rust converter input thick polylines before output flows.
   Use external GDB, without production logging or coordinate injection.
3. If input differs, move one boundary upstream and revise the source owner
   before any implementation. If input matches, inspect split/group arithmetic.
4. Add a public complete-output regression before the minimal demonstrated
   repair, then rerun the original KSR and affected consumers.
5. Run all standard gates and independent six-axis review; repair/re-review
   before combined code/docs commit and push.

Ranked falsifiable hypotheses:

- Arachne input contours differ: matching corresponding generated domains
  should remove the junction and downstream flow difference.
- Same contours produce different width junctions: converter inputs already
  differ, so identical converter arithmetic cannot fix the root cause.
- Same thick polylines convert differently: native/Rust input coordinates and
  width bits agree, but split/group output is the first differing boundary.

Existing strict RED63348 completed100/18.714s. Full artifacts are
`/tmp/ares-ard76-ksr-ordered-current/case-MGIbJ2`; original project SHA256
698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9.
Current and pre-ARD76 Ares raw streams match exactly, SHA256
574fc84db8f2a0e081e9f2db172f187a7d79804243c151c09c5e3a5976e5dbb9.
Full comparator remains RED at line9946 M73 timing; semantic diagnostic
separately identifies the16 internal-solid moves on layer49.

Native reference remains the approved seed/fill/object-ID initialized build,
whose wrapper/ELF hashes were revalidated after recovery. GDB was missing
from PATH and the store; restoration48855 completed0 using the same fixed
Nixpkgs, retained through `/tmp/ares-ard77-gdb`.

Native probe15273 stopped at a Python type-lookup exception (debug directory
set after automatic initial symbol loading), not a completed slice; corrected
probe explicitly reloads symbols. Native30435 completed0, Rust33703 completed0
with full G-code retained. Native input at Fill z9.8/layer48 and Rust input at
generate_layer index48 each contain86 thick polylines.84 match in coordinates
and every width value; two already differ before variable-width conversion.
There is no width mismatch among those84 corresponding identical geometries.
Native/Rust logs are `/tmp/ares-ard77-native-width-corrected.log` and
`/tmp/ares-ard77-rust-width.log`. The major unmatched path shares five inner
vertices but has native endpoint(-12948843,-10240203),width584726 versus
Rust(-12964813,-10241430),width587174; the second path differs by one unit at
one Y coordinate. These are diagnostic observations, not permanent assertions.

Initial Rust input-domain breakpoint115/116 was mislocated on the error-only
closure at116 and captured no domains, so native/Rust domain equality is not
yet established. Corrected Rust probe moves it to the successful call at115;
no production code is changed. Converter-only hypothesis is deprioritized;
contour and junction hypotheses remain open. Original-input minimization to a
public small-model reproduction remains required before a production repair.

Corrected Rust24816 completed0, log
`/tmp/ares-ard77-rust-width-corrected.log`. All44 native input contours/holes
match all44 Rust Arachne inputs exactly, including vertex order. This rejects
the contour-drift hypothesis for this layer. The converter-input mismatch is
between matching Arachne input and later thick polyline materialization; next
capture generated junctions before/after toolpath postprocessing, rather than
changing the converter. The main differing path belongs to native domain
call8 with30 contour vertices and no holes, a candidate small input to minimize.

Targeted stage probes remain diagnostics, not acceptance runs. Initial
native95006 stopped when an inlined simplify breakpoint no longer retained the
entry ABI register; corrected native14996 uses verified machine-entry symbols
for stitch/removeSmallLines and typed DWARF for inline simplify, and completed0.
It captures2 lines/66 junctions before and after both stitching and short-line
removal, plus the21-vertex prepared outline at skeletal-generator entry.
Rust36527 completed0 and captures3 lines/70 junctions before small-line removal,
2/68 after removal, then2/26 after simplification. These stages differ already
before small-line removal; width conversion is not the first divergence.

Rust prepared-outline probes62223 and96157 stopped on a wrong variable name
and an optimized-out inlined argument respectively, not completed captures.
The next probe reads the owned prepared vector in generate's caller instead.
Native records `/tmp/ares-ard77-native-prepared.log` and original Rust stages
`/tmp/ares-ard77-rust-stage.log` remain retained. No production repair yet.

Caller probe42672 completed0; log
`/tmp/ares-ard77-rust-prepared-caller.log`. Both breakpoint locations at the
caller capture the same21-vertex prepared polygon, exactly matching native
skeletal-generator input in coordinates and order. Target stage comparison:
native stitch-before/after and small-before/after each2 lines/66 junctions;
Rust stitch-before/after each3/70, small-after2/68. Thus divergence already
exists before stitching, with identical prepared contours. Revised spec now
cites SkeletalTrapezoidation construction/generateToolpaths and existing Rust
trapezoidation modules; a converter or outline repair would be misaligned.

The tempting repeated local-maximum predicate in Rust filter_central is also
present verbatim in upstream filterCentral. Do not independently correct that
upstream behavior as a parity fix. Next steps are a small public model from
the30-vertex input and staged graph/junction comparison inside the identified
generator boundary. Also compare constructor config and beading-strategy
parameters: polygon equality alone does not establish all generator inputs
are equal. If caller parameter construction is the first mismatch, revise
the source owner again before implementation. No accepted root-cause repair yet.

Constructor/factory probes now exclude parameter drift for the target domain.
Native `/tmp/ares-ard77-native-params-valid.log` captures angle
0.1745329350233078, step800000, filter distance100000000, deviation100000,
propagation400000; the Rust caller config matches those values. Native
`/tmp/ares-ard77-native-factory.log` and Rust
`/tmp/ares-ard77-rust-skeleton.log` match all13 beading-factory inputs,
including split/add thresholds0.61904996728480777/0.80952498364240388.
Both slices completed normally with complete G-code retained. Earlier native
params launch attempts used the wrapper executable or an unsupported
--export-gcode flag; neither reached slicing and neither is parity evidence.

Initial active-node multisets also match:107 points and boundary distances in
native `/tmp/ares-ard77-native-skeleton.log` and Rust
`/tmp/ares-ard77-rust-nodes.log`. Rust retains138 storage slots including31
deleted nodes; comparing raw storage lengths would incorrectly diagnose a
construction mismatch. This proves node coordinate/distance equality only,
not adjacency or traversal order. Half-edge connectivity capture is next;
if that agrees, compare successive generator stages before blaming Voronoi
construction. No production repair or new comparison tolerance is justified.

Half-edge captures95443/40224 completed0. Native/Rust each have212 active
edges with matching multisets of from/to/next/prev/twin coordinates; external
`/tmp/ares-ard77-compare-edges.js` reports no unmatched entries. Independent
review accepts that limited conclusion, not node identity or traversal-order
equivalence. Coordinate duplicates can hide identity differences. Subsequent
parser `/tmp/ares-ard77-compare-stages.js` checks contiguous Rust slots, all
node/edge references and successful enum parsing rather than silently using
null when parsing fails.

Stage captures93083/23026 completed0, logs
`/tmp/ares-ard77-native-central.log` and
`/tmp/ares-ard77-rust-central-symbol.log`. Initial, post-updateIsCentral,
post-filterCentral and post-updateBeadCount snapshots each match107 active
nodes (coordinates, distance, bead_count, transition_ratio) and212 active
edges (coordinate connectivity, edge type, central flag). The native recursive
filterNoncentralRegions entry repeats; comparison uses its first entry only.
The first demonstrated mismatch is still later than these matched snapshots.
Next inspect filterNoncentralRegions and transitioning ribs, preserving the
remaining identity/traversal-order caveat.

Rust source-line probes5456/54098 completed slicing but mapped lines81/82 to
line85 after inlining, so their filtered-central/bead-count labels are invalid
and excluded. The corrected probe uses verified function-relative offsets
1739 (before bead-count update) and3262 (before noncentral filtering loop),
with C expression parsing only while constructing those breakpoints. Earlier
mangled-name and Rust character-literal attempts57856/68724 failed before
slicing. External machine offsets are throwaway diagnostics, not source-level
pinning tests and not shipped in the repository.

All successful probes preserve full G-code, but none is a strict parity GREEN.
The fresh native stream still differs from the stored initialized reference
in M73 placement and estimated model time. Do not substitute the debugger
output as a replacement golden or normalize away those differences. No Rust
production code was changed during these parameter/graph diagnostics.

Transition-stage probes9381/34889 completed0 with native/Rust full output
retained at `/tmp/ares-ard77-native-transitions-output/plate_1.gcode` and
`/tmp/ares-ard77-rust-transitions-corrected.gcode`. Logs with the same prefixes
capture matching107-node/212-edge state after noncentral filtering, then
native110 nodes/218 edges versus Rust113/224 after transitioning ribs.
Rust-only node(-12964813,-10241430), distance670666, bead_count3 is exactly
the previously observed divergent thick-polyline endpoint. The other two
Rust-only nodes are boundary endpoints of the inserted ribs. This associates
the downstream difference with this stage, not a proved faulty subfunction.
Native/Rust prepared contours and earlier captured states remain equal.

Rust probe51021 stopped at an external Python attribute error before graph
capture; GDB returned0 without a completed inferior slice and is excluded.
Corrected34889 initializes each snapshot's one-shot guard. Rust offsets3484
and12354 are verified after the noncentral loop and before the first extra-rib
loop body; one-shot guards avoid reading later loop iterations as stage entry.
The same parser reports zero multiset differences through noncentral-filtered
and the three-node/six-edge excess at transition-ribs. Next split that stage
into middle generation/filtering, end generation and insertion; retain the
required public small-model RED before changing production code.

Independent stage review confirms the boundary, with an important precision:
the edge total grows by6, but records contain6 native-only/12 Rust-only edges,
including existing connectivity replacement, not merely six appended edges.

Native52887 completed0. Rust84495 completed0, full native/Rust streams retained
under `/tmp/ares-ard77-native-transition-payload-output/plate_1.gcode` and
`/tmp/ares-ard77-rust-transition-substages.gcode`. External snapshots compare
generated mids, filtered mids and ends before application; each contains the
same single owning edge and identical lists. The edge has middle
(168396,3,682333), ends(16016,3,true)/(416015,3,false). Recorded graph state
matches through those boundaries, then differs after application. This moves
the candidate owner to applyTransitions and graph insertion, not middle/end
generation. Rust offsets5829/7401/8975 precede their first loop bodies, verified
in the existing generator disassembly. Ordered transient lists are preserved;
post-application weak-storage lifetime differences are not compared as output
requirements. The prior Rust90968 slice completed normally but the debugger
command file was edited while its run was active, causing a trailing command
parse error and terminal1; keep that record distinct from clean84495.

Candidate source mismatch: native applyTransitions caches original from/to
before processing all ends; Rust apply_transitions_on_edge re-reads the edge
endpoint for close_node on each iteration, although insert_node mutates it.
Prediction: when the far end is inserted first, a later endpoint within snap
distance of an original node compares against the inserted node's bead count,
creating the observed extra node. Capture actual endpoint processing order
and cached versus current node identity before a repair. Public minimized
complete-output RED and root-cause regression are still required.

Snap probe62126 completed0, `/tmp/ares-ard77-rust-snap-check.log`. Normalized
edge103 length509278 processes positions93263/requested4 then493262/requested3.
The first insert creates node138 at(-13363638,-10272051), bead_count4. The
second snap check selects that node rather than the captured original finish
(-12948843,-10240203), bead_count3. This confirms the mutated-endpoint cause.

Public minimization now reproduces through a valid closed30-vertex prism,
116 nondegenerate triangles with each undirected edge shared twice. Native
export applies the11 requested settings, then slice produces3 layers. Cached
strict replay completes100/0.435s at
`/tmp/ares-ard77-minimal-red/case-BwvxI1`, first difference line244 width
0.388839/0.389043 and subsequent G1 coordinates/E values, not metadata/M73.
The normal live-reference regression50568 completes100/1.449s with the same
symptom at `/tmp/ares-ard77-transition-test-red-valid/case-I1kQkp`.
Earlier test attempts failed to compile a missing explicit child-mod path or
failed reference initialization due to using ORCA_REPLAY_BIN rather than
ARES_ORCA_BIN; those are not parity REDs. No production fix preceded this RED.

Minimal implementation reuses the cached from/to NodeIds for close-node
selection, preserving native applyTransitions behavior. Remove the obsolete
private task22o173 expectation that demanded the mutated source endpoint;
new public test checks complete output, not private graph layout. Regression
12563 completes0/1.460s, full artifacts under
`/tmp/ares-ard77-transition-test-green`. The comparator is unchanged. Source
test modules contain67/57 lines, registry387; production apply module94 lines.
Fmt/check and diff/check pass. Clippy42551 completes0; Tier-1 core checks and
the WASM adapter check84457 complete0. Original KSR strict replay51455 remains
RED100/19.874s at M73 line9946, full artifacts
`/tmp/ares-ard77-ksr-ordered-current/case-9HVuI9`. Workspace96655 completes100:
5692 passed, one KSR semantic diagnostic failed, nine skipped and1765 not run
because of fail-fast. The first semantic difference advances from layer49 to
layer64/Z12.8: nine unmatched expected internal-solid moves versus14 actual.
This proves progress at the repaired boundary, not complete KSR parity.
Six live complete-output consumers96132 complete0/1.841s, artifacts
`/tmp/ares-ard77-public-consumers`; bridge, rehash, compressed, tilted mesh,
short-line and transition-snap cases all pass. Independent finite pre-review
finds no required code repair; final and combined approval remain pending.

Independent final finite review approves the cached-endpoint repair, with no
required implementation/test items; combined commit/push remains unapproved.
No-fail-fast workspace59915 completes100/258.677s:7457 passed, one failed,
nine skipped. The same KSR layer64 diagnostic is the only failure; there is
no longer an unrun-test gap. Next diagnostic captures3075/48306 complete0
with normal inferior exits and full output under `/tmp/ares-ard78-*width*`.
Layer64 input contours agree (six each), while two of12 converter input
polylines differ. Stage captures29508/86461 complete0: all six pre-stitch
paths match, only two of four post-stitch paths match. Small-line processing
does not introduce the discrepancy. This residual requires a separate
`PolylineStitcher.hpp::stitch` / `SparseGrid` boundary investigation, not
expanding the verified applyTransitions repair.
