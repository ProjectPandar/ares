# Stage2 explicit-height wall-loop range verification

Status: ready for coordinator review; not merge approval.

The first range-dependent PrintRegion slice ports PrintApply.cpp::LayerRanges /
generate_print_object_regions / get_create_region, PrintObject.cpp::
region_config_from_model_volume, and PrintObjectSlice.cpp::slices_to_regions.
Core keeps an object-wide config-distinct region table and per-range volume
mappings; slice Z chooses the mapping. Existing single-range scaffolds are
replaced, not retained as fallbacks.

The unchanged runtime Native input has 50 physical layers at constant 0.2 mm
height and a wall_loops override from 2 to 3 inside the explicit-height 2–6 mm
range. The initial runtime RED rejected layer_config_ranges. After routing was
implemented, whole-output comparison exposed the coStrings prerequisite and
then the incorrect region-major template layer count. The merged generic
coStrings correction removed the former blocker. Export templates now obtain
total_layer_count from the physical export schedule, matching GCode.cpp:
2487–2529 / 2890 rather than counting region records.

Workspace testing exposed a real zero-height regression: empty physical layer
lists reached region-major traversal before the legacy export error. The fix
restores PrintObjectSlice.cpp:866–867's slice-completion rejection before
perimeter traversal. The unchanged regression assertion receives the original
`no layers were detected` error; no step-size special case was added. The
failed attempt remains non-PASS evidence, separate from the corrected run.

The corrected original wall-loop witness passes complete Native/Ares output
comparison. Only the entire generated-by line is excluded. Motion, M73,
configuration, layer counts, and template output are not normalized. Stage1
height controls and the spiral region-range rejection control remain intact.

Fresh seeded pairing after the cooling and seam source changes passes 743
baseline and 745 candidate tests, with no failures or skips. Input-bound audit
finds 623 actual common Ares streams byte-identical across 606 input identity
groups, no unmatched baseline or ambiguous groups, and one new Stage2 stream
fully equal to runtime Native. Final required-Native all-feature workspace
passes 8132 tests with seven existing skips. Workspace all-target/all-feature
clippy with warnings denied, core/WASM all-feature wasm32 check, and rustfmt
also pass. Subsequent coordinator-authorized test-only changes carry this
evidence forward without claiming that old runs executed on newer commits.

This witness does not establish parity for every region option, modifiers,
multifilament, painted profiles, SlicingAdaptive, or active ZAA. Spiral region
overrides remain unsupported. PrintObjectSlice.cpp:244–266 epsilon-aware range
selection is a separate deferred RED/GREEN slice and must not be folded into
this implementation commit.

Exact source/input identities, run-to-commit bindings, commands, full generated
streams, failed attempts, and audit records remain outside the repository for
coordinator review. No frozen Native G-code or process receipts are committed.
