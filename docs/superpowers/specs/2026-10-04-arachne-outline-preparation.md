# Arachne outline preparation complete-output parity

Source files/functions, Rust destination and included/deferred behavior:
ARD-0100. Replace only established gaps in the upstream outline preparation
rewrite; do not grow an independently designed Ares wall pipeline.

Public seam: unchanged saved or freshly exported3MF through Ares to complete
G-code, compared with official2.4.2 and separately seededCPU0 reference.
Retain existing hole and actual-overhang strict assertions. Verify missing
corner/open-wall and extra tiny-loop symptoms without changing the fixture
or normalizing away real movement. Legal wall_maximum_deviation domains are
0.005..0.05 (PrintConfig.cpp:7253–7263), never an epsilon<1 bypass.
The established input prerequisites include wall/top-surface floating height
and degree-to-radian conversion. Cover wall_transition_angle1..59 degrees
(PrintConfig.cpp:7195–7206):min,max,seed0x5eed LCG interior, verified in saved
configuration; native boundary extrusion moves must differ, not only footer.

For each implemented small slice: sensitive RED first, complete-output GREEN
on its targeted case, prior finite GREEN controls unchanged, workspaceNextest,
Clippy baseline/fmt/WASM compile, independent six-axis review/fix/review.
Commit code/tests/docs together and push the matrix branch, not main.
Whole-printer/Option completion remains a separate open goal.
