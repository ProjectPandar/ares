# ARD-0065: Bottom-shell scalar range parity

Upstream owner: `PrintConfig.cpp:1156-1165` declares `bottom_shell_layers`
as coInt, min 0, without a smaller explicit ceiling. `ConfigOptionInt`
(`Config.hpp:955-983`) has the signed scalar domain, including INT_MAX;
`ConfigOptionDef` (`Config.hpp:2475-2479`) supplies a larger FLT_MAX ceiling.
The effective representable range is therefore 0 through 2147483647.

`PrintObject::discover_vertical_shells` (`PrintObject.cpp:2252-2278`) projects
bottom regions through actual lower-layer indices. Horizontal propagation
(`PrintObject.cpp:3983-4020`) likewise uses layer distance and the requested
count. Rust destinations already include `prepare_infill/vertical_shell_projection`
and `horizontal_shell_propagation/window.rs`, with the existing byte-output
slicing boundary as the production verification seam. The native option
fixture is a temporary adapter around these upstream contracts.

Include scalar min/max and a fixed-seed interior spanning the full range in
`option_coverage/domains.rs`; test through original export, exact applied-value
proof, final config literals and complete ordered G-code in a separate
`bottom_shell_count.rs` test module. Preserve the requested huge count even
when the model has fewer layers. Require zero bottom-shell thickness in this
fixture and distinct emitted motion for active counts versus zero to show that
the option affects slicing. This activation comparison does not replace or
relax the complete-output comparator.

Defer top-shell counts, spiral-mode and nonzero-thickness cross-products,
multi-region combinations and other implicit ranges. In particular, top-shell
vertical projection has a separate signed `idx_layer + n_top_layers` expression
(`PrintObject.cpp:2227`), which needs its own original-binary investigation at
INT_MAX. Production changes require observed discrepancies and remain within
the cited shell rewrite boundaries.
