# Arachne overhang reversal complete-output parity

Source files/functions, Rust destination, included/deferred behavior and
temporary-shell limitations: ARD-0099.

At the user-approved public 3MF-to-complete-G-code seam, preserve unchanged
Prusa MINIIS stock settings with Arachne and overhang_reverse enabled. Add
native-sensitive reversal controls that verify effective saved configuration
and real second-layer extrusion movement changes, not merely footer changes.
Cover both Boolean values and internal-only/threshold/raft boundaries through
public input slices as the implementation progresses. This is an upstream
rewrite slice, not an Ares-owned wall pipeline.

Acceptance: sensitive RED before production changes, strict complete-output
GREEN for saved-original and fresh native controls, independent six-axis
review/fix/review, workspace Nextest, no additional Clippy warnings, fmt and
WASM compilation. Commit code+tests+docs and push each verified checkpoint.
This does not close all printers or all option domains.

User-approved finite checkpoint: save the independently verified crossing
widths, shared PointHashMap and their consumers before outline/raw work.
Saved-original and finite fresh controls must pass; default-thread failures
must be preserved and checked against unchanged baseline output. Retained
hole/actual-overhang RED and deferred raft coverage prevent claiming this
whole slice complete, but do not defer the requested checkpoint indefinitely.
