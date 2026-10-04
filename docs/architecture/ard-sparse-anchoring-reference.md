# ARD: sparse anchoring shares the PrintObject-centered reference

Upstream ownership: `Fill.cpp:1394-1442` sparse-anchoring generator sets its
filler bbox from `PrintObject::bounding_box` (`Print.hpp:347`); the latter is
symmetric, so `Fill::_infill_direction` (`FillBase.cpp:286-290`) uses zero.
This is the same contract as normal fill generation, integrated in94a0a629.
Compensated or discretized slice contours must not redefine the anchor center.

Change only `sparse_anchoring/mod.rs` reference selection. Remove the now-dead
slice-center helper and re-export instead of retaining a compatibility fallback.
Existing Ares scaffolding otherwise remains a temporary shell around upstream
sparse fill generation. In particular, extended box dimensions are deferred,
not silently claimed equivalent. Bridge math/flow and extra-overhang guards
are outside this slice. This adds no new Ares-owned pipeline or runtime coupling.

Complete ordered G-code is the regression contract; diagnostics must never turn
into tolerance or filtered-output acceptance. Preserve the16-case RED and all
residuals. Source-only correctness is insufficient to claim parity improvement.
See the matching spec/plan for case inventories and gate receipts.
