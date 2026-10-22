# Front-edge nearest: initial numeric generator cursor

## Status and source ownership

SPEC only, based on main92238c87148e9e2fe42c355ac76b60a27c527a0d.
This is a source-cited Rust rewrite slice, not an Ares-designed pipeline.
No production implementation is authorized by this document. Public RED follows
in a separate test/input-only commit; GREEN/FULL require coordinator approval.

Upstream pin8500fcdccaa10b5099ac20d252af3a7c560046f1:
- libslic3r/Point.hpp:277 default Point3 numeric zero;
- GCode.hpp:197,224,397-399,607-608 numeric m_last_pos independent of
  m_last_pos_defined; setters define the cursor, getter reads its numeric value;
- GCode.cpp:5659-5669 GCode::set_origin always translates numeric m_last_pos,
  independently of its defined flag, and translates the wipe path;
- GCode.cpp:5769-5773 extrude_loop passes last_pos to seam placement;
- GCode/SeamPlacer.cpp:1554-1558 spNearest consumes that numeric preferred point.

Rust destination is the generator state boundary:
project_slice/gcode_emit/motion/state.rs last_scaled_position,
motion/origin.rs::set_origin, motion.rs::local_cursor and their direct
numeric-cursor/defined consumers. Existing Option-based state is only a temporary
compatibility shell for this upstream concept: None currently conflates the
existence of a numeric cursor with its defined-for-travel status.

## Non-negotiable rewrite decisions

1. Preserve a numeric scaled generator cursor from initialization at zero,
   independently of whether upstream considers it defined for travel.
2. Translate the numeric cursor for every set_origin, even while undefined,
   with the existing upstream scaled-rounding order. Keep wipe translation intact.
3. Seam/ordering reads consume that numeric cursor, not a reconstructed writer
   XY minus an offset. Writer position and generator cursor remain distinct.
4. Existing transitions that set or invalidate the cursor must preserve the
   source numeric/defined distinction; invalidation must not silently erase the
   numeric value. This is not permission to change travel/lift decisions.
5. No fallback to writer coordinates for undefined numeric state; no feature
   flags, legacy compatibility branch, special-case fixture or cached answer.
6. Keep integer cursor arithmetic/rounding and deterministic iteration, with no
   new per-path allocation or dependence on worker count. All changed text<400LOC.

Included: source cursor initialization, unconditional origin translation and
faithful numeric/defined state plumbing required by existing consumers.
Deferred: seam candidate geometry/scoring/visibility, negative occlusion, region
Boolean, painting, routing, retraction/lift policy, processor/time, print_order,
and the separately frozen volume-affine and other approved chains. A mechanical
consumer migration is not authorization to redesign those consumers or loosen
assertions. If the minimal owner correction cannot pass the witness unchanged,
STOP and report rather than patching downstream scoring/travel/time.

## New qualified positive-only witness

Input-only Native exports use two normal parts, Classic/nearest, no negative
volume, scarf, fuzzy skin, EFC, skirt/brim or arc fitting. Common GUI instance
scale X100%,Y10%,Z40% and placement give world mesh bbox[25,0.5,0]..[35,1.5,4].
Only exported GUI setting difference is extruder_offset["0x0"]->["0x2"].
PrintConfig.cpp:2305-2313 declares the advanced mm option;
slic3r/GUI/Tab.cpp:5134-5135 exposes it in the Position group.
All other ZIP member contents, model/build/part/config payloads are identical.

Control SHA256:
0f5ff07e6145155275ba82bbedfd608569117a3ee30a26309686efba2764f0c1
Target SHA256:
d70b49101439c4a040cad4bbba3baf728d0d7d39d97963e59718ed245a5087db

Discovery: control official+seeded complete byte PASS; target official first
non-M73 difference byte66821,line1342,column13, expected93455/actual93483 bytes:
Native G1 X25.25 Y-1.25 versus Ares G1 X25.25 Y-.75. Actual API/CLI raw equal.
Target seeded comparison was not run after that strict STOP; Native2 was already
stable in both modes. This is not a seeded-target verdict or a formal RED claim.

Bounded read-only owner audit captured all four complete observers normally
exiting, outputs unchanged, no inferior calls/writes or invalid probes:
- Native both: numeric[-30000000,-1000000,0], definedfalse, origin[30,1],
  writerXY[0,0], preferred[-30,-1];
- Ares control: None, offset[30,1], actual preferred[-30,-1];
- Ares target: None, offset[30,-1], extruder_offset[0,2], writerXY[0,0],
  origin[30,1], actual preferred[-30,+1].
Five ordered first-loop points/candidates, range[0,5), angles, overhang and
embedding match exactly. Native LIVE visibility is1 throughout; Ares LIVE cached
base1.5264784097671509 equals HOST angle-plus-visibility arithmetic. HOST replay
changing only preferred reproduces corner2 versus1; raw runtime index and Native
expf bit arithmetic are NOT claimed captured. Native DWARF was seeded debug,
not official. Evidence is an input-owner proof, not validation of a code fix.

Discovery root: /home/indexyz/ares-evidence/w6-92238-front-edge-nearest-br884s6n/
Audit root: /home/indexyz/ares-evidence/w6-front-edge-owner-audit-pvpa3_kb/
The obsolete489/66/dd8 initial-cursor chain remains discarded. No ancestry,
source patch, test migration or causal claim from that chain is inherited.
The earlier negative-volume witness did not qualify cursor behavior; this new
positive/front-edge pair and new audit are the sole evidence for this slice.

## Public RED and later exit gates

Use runner::native_stream_case::check -> NativeCase directly, signature
(false, |_, _| {}), no copied comparison engine. Freeze the two input-only
archives and SHA. Native2 stable FIRST precedes real byte API/CLI, whole ordered
generator-only comparison retains M73, time, motion, comments, metadata/footer.
No skips, expected rejection, tolerance, guard or Native-derived expected stream.
Target should genuinely fail, control pass in each authorized directed mode.
Independently audit actual raw API/CLI equality; target canonical panic may
precede final raw assertion/adapter update and must be disclosed as such.

RED RR includes both-mode receipts, clean scope/ancestry and input/SHA audit,
fmt and strict workspace/all-target clippy, all changed text<400LOC. No production
change or unit-test migration in the RED phase. After explicit GREEN approval,
verify unchanged public pair first, then authorized adjacency/wasm compile/FULL
on the exact final tip; never credit an earlier or later integration tip.
Integration window instructions preempt this independent work. No rebase/merge/
push, frozen-chain modification or callback adaptation is implied by this SPEC.
