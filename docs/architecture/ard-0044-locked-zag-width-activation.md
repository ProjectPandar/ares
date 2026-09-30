# ARD-0044: Activate Locked Zag width comparisons

The upstream boundary is `libslic3r/PrintConfig.cpp`'s
`skin_infill_line_width` and `skeleton_infill_line_width`,
`Fill/Fill.cpp::group_fills`'s `ipLockedZag` flow construction,
and `Flow::new_from_config_width`. The existing Rust implementation lives
in `ares-core/src/project_slice/group_fills/params/locked.rs` and the
existing fill/emission modules; this slice changes only the CLI parity
activation helper and its public project-to-G-code tests.

Both width probes must select `sparse_infill_pattern=lockedzag` with
nonzero sparse density. A comparison under the default crosshatch pattern
does not exercise these flow parameters. The test must verify exported
settings, positive sparse extrusion at the resolved width, and the complete
ordered output under generator-only normalization. Configuration comments
alone are not an activation witness.

Include the existing authored 0.3 mm and 150% probes on the 0.4 mm-nozzle
cube context. Preserve the original oracle, baseline, comparator and
domain-completion policy. Raw/context endpoint expansion, other Locked Zag
options, untouched profiles and cross-platform runtime parity remain
deferred; these two sampled domains remain INCOMPLETE. Existing scaffolding
is temporary wherever it does not match the cited upstream boundary, not
a basis for a separately designed Ares pipeline.
