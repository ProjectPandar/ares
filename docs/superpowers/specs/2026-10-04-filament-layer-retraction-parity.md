# Effective filament layer-change retraction parity

Source boundary, Rust destination, included/deferred scope and scaffold
limitations: ARD-0097. The public seam is native exported 3MF bytes to
`slice_project` complete G-code, the same seam requested for option coverage.

Every `filament_retract_when_changing_layer` value (false, true, nil) must
remain effective in native export and pass the unchanged whole-output
comparison. Correct the production consumer to read the already resolved
runtime configuration rather than the full machine defaults.

Include the saved public sweep failure as a before/after case. Defer unrelated stock bounds/unsupported-feature and other
option-domain gaps; they remain mandatory future source-cited slices.

Acceptance: sensitive RED/GREEN complete domain, official and seeded single-CPU
native references with default-thread official control, workspace Nextest,
fmt/Clippy and WASM compilation. This is not global completion.
