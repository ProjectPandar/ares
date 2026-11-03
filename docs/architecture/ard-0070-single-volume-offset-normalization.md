# ARD-0070: Single-volume import offset normalization

Source boundary: `Format/bbs_3mf.cpp:2187-2207` in upstream 8500fcdcc.
After importing volumes and instances, the importer compares the first
instance offset to its composed volume world offset with Eigen `isApprox`.
For a single volume whose offsets differ, it moves the composed offset to
every instance and clears the volume translation. Linear transforms remain
unchanged. Adjacent multi-volume, shared-mesh and assembly-pose handling is
not changed by this slice.

Rust destination: a small `project/domain/volume_offsets.rs` module invoked
by `project/load.rs` after domain assembly. Existing project structures are
temporary validation shells around upstream ModelObject/ModelVolume concepts;
this is importer parity, not a new Ares slicing pipeline.

Preserve world placement and public instance identities, metadata, volume
linear transforms, source provenance and mesh sharing. Use the upstream
relative squared-norm `isApprox` contract and double precision, not an
arbitrary epsilon or unconditional shift. Zero/multi-volume objects do not
enter the single-volume normalization branch. Internal assembled objects
already have instances; do not add speculative internal validation.

Evidence: the complete short-branch prism replay is RED at line 273
(`0.394158` versus `0.394156`), retained in
`${TMP_ROOT}/ares-concentric-prism-three-field-corrected.OkE1nK/case-QlGglr`.
Native and Rust debugger captures show different fill input contours before
Arachne. No native small line is removed in this case. All 110 transformed
mesh vertices differ; the native composed translation is
(-0.596005, -0.901509) mm versus Rust
(-0.5960050534797059, -0.90151050759456) mm.
The corrected native ABI probe reads the matrix from rcx, not rdx: the
compiler split the mesh argument into begin/end pointers. The earlier
matrix capture is invalid evidence; its returned vertex capture is valid.

Public seams: `load_project` observable model/instance transforms and the
existing project bytes-to-complete-G-code native parity harness. Require
independent native output, active imported offsets, sensitive full-stream
RED/GREEN, preserved exact inputs/outputs, and counterfactual reversion.
Do not inject captured native vertices or normalize extra output to obtain
GREEN. A debugger-only vertex replacement removed the four original numeric
differences but exposed two travel-coordinate differences; it is diagnostic
evidence, not a fix or accepted parity result.

Keep official, seed-only and all initialized references distinct. This
slice uses the seed/fill/object-ID reference identity from ARD-0069; it
does not close concentric short-line filtering or all-device/all-option parity.
