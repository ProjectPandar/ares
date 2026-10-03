# ARD-0101: Support Option complete-output evidence

The source domains are OrcaSlicer v2.4.2 `PrintConfig.cpp:6061` (support_type)
and `:6355` (support_style). Runtime ownership remains `SupportMaterial.cpp`,
`TreeSupport.cpp` and `TreeSupport3D.cpp`; their Rust destinations are the
existing `ares-core/src/project_slice/support_material` modules. No new Ares
slicing pipeline or support algorithm is introduced by this acceptance slice.

Normal and Tree are connected. The CLI Option sweep's old blanket
unimplemented-support classifier is obsolete, not a runtime contract.
Every actual full-output comparison must count, including DIVERGENT; its
first real mismatch must remain a FAIL. Do not relax comparators, replace
public inputs or reclassify incomplete domains as complete.

Included: remove the obsolete support classifier/counter and its private
pinning assertions; test the actual sweep subprocess and persisted report
against all four public support_type values and their paired manifests.
Deferred: fixing actual Tree output differences, other Option legality gaps
and old nondeterminism classifications. They remain open, not waived.
