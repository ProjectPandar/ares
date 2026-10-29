# S-C0 source correction and directed evidence

## Source and commit order

The frozen public RED is c15b29c60 (approved; merges only with GREEN).
The first unequal intermediate was resolved before the downstream hook:

- 194256066 ports the missing sparse-anchoring entry inset from
  PrintObject.cpp:2744-2754 / Fill.cpp:1394,1440-1453,1492-1498 /
  FillBase.cpp:105-117. Anchoring uses the constructor's overlap0 and a
  completed double scaled delta followed by one f32 cast, then all Miter3
  offset components. Ordinary append and W6's octree work are unchanged.
- da9b24275 ports the observed missing boundary collision input and the
  corresponding FillAdaptive.cpp:654-668/670-778/1034-1044 hook query semantics.
  The actual contour and holes accompany the source line segments. Query
  coordinates complete through float, only source infill receives thickness
  clearance, and the finite-query reverse/other-hook rules are preserved.
  Hook-length completion follows1398-1399; displacement casts precede integer
  addition at767-768. The existing LineDistanceTree accelerates finite segment
  queries; their minimum-distance reduction does not depend on hit order.

The touching-grid assertions remain. No panic is caught, no input is skipped,
and no admission guard, fallback, unsafe or suppression was added. Existing
hook test assertions remain; their calls now provide an authored boundary
for the source-required argument. The public test comment describes evidence
capture order without a temporary-state narrative.

## Runtime causal observations

The read-only inset observation is retained at
`/home/indexyz/ares-evidence/worker3/support-touching-grid-inset-observation/20261015-130402`.
At194256066, first anchoring Z9.2 and normal Z0.8 both receive the Native
inset domain min(-9019688,-9019688), max(9019693,9019693). S-C0 still failed
both modes at that point, satisfying the conditional hook-extension gate.

The release Native hook observation is retained at
`/home/indexyz/ares-evidence/worker3/support-touching-grid-hook-boundary-observation/`.
The actual forward-query result is one top contour segment, id16 with15
source lines, from(9019693,9019693) to(-9019688,9019693). No other hook was
present for this query. Its collision limits forward to621767.7732422437;
backward remains1628318.5. Ares's source-only scan omitted that boundary and
retained the full forward length. This observation identifies the missing
candidate; it is not an equal-length direction-choice explanation.

## Public parity and final checks

Exact input SHA256 remains
9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1.
At production commit da9b24275, the public S-C0 test passes official and
seeded on CPU12, using runtime Native, strict whole-output comparison and
raw API==CLI equality. Both Ares outputs have SHA256
2aabf366c7533c3981e8ce2c21122fd4561af5393cd6e78ef9efb6972a6c0be2.
No Native output or expected geometry is committed.

Durable directed evidence is recorded under
`/home/indexyz/ares-evidence/worker3/support-touching-grid-green/`.
The final REVIEW-REQUEST and its evidence JSON identify the exact final
head, control selection/counts, gate exits and artifact hashes. They must
include B-S's ten-case matrix, connector/anchor/Adaptive controls and
fmt/strict/wasm32 results; this source-observation record alone does not
establish those remaining checks or full/workspace approval.

Honeycomb/TPMS-D anchoring omissions remain deferred source-only candidates.
W6's build_object_octree/octree.rs boundary and mechanical octree_for '?'
change are untouched; the later landing chain rebases and reports conflicts.
Object-region spacing stays blocked until this slice lands; the earlier
Inner-wall finding remains deferred. No push is authorized.
