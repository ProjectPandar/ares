# ByObject ordinary owner brim prerequisite — public RED

Source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Source ownership: `Print.cpp:2465-2467,2520-2530` sequences instances while
collecting distinct owners; `Brim.cpp:36-45,573-581,914-977` builds and infills
ordinary all-instance owner areas; `GCode.cpp:3230-3243,3280-3345,4486-4531`
initializes owner IDs once, prints sequential instances, emits each brim owner once.

Input-only frozen 3MFs and SHA256/member manifest. Two owners, three instances:
object2 at X50/X125, object3 at X200; Y110,Z2, Z scale0.4. ByObject, outer_only
brim8/gap0, combine_brims0, combined skirt2/height1, single filament, no
support/raft/tower. `combined.3mf` is the accepted failing primary target.
`no-skirt.3mf` differs ONLY in skirt_loops0 and exposes a direct brim path
mismatch. `control.3mf` differs from combined ONLY in brim_width0 and must pass.
Meshes, placements and other ZIP members/settings are unchanged among all three.

Native is generated FIRST at runtime; no Native output is committed. Tests assert
Native extrusion at all three model placements, two sequential origin changes,
and, for active brim cases, positive brim extrusion at all three placements in
TWO owner blocks (rather than the current Ares THREE per-copy blocks). Combined
has one skirt block, no-skirt has none, width0 control has no brim and one skirt.
All cases require validated API/real CLI success, raw API==CLI, and unchanged
complete `golden::compare_ordered_bytes_generator_only` equality to Native.

Expected authentic RED in both modes: combined fails
at byte3509/line123 (Native skirt G1, Ares M73); no-skirt fails at byte1607/line67
(Native first brim point X114.334/Y100.314, Ares X42.008/Y98.284). Width0 control
passes complete ordered equality. No inverted assertion, M73 filtering,
comparator relaxation, tolerance or estimator change. Do not assert divergence
as success. Preserve the ByObject perobject-skirt guard (stage B deferred);
this RED exercises only the ordinary-owner prerequisite A.

Use separate fresh official/seeded cache and artifact roots and the canonical
reference environment. Any residual whole-output divergence after the faithful
owner port means STOP/report, not estimator repair.
