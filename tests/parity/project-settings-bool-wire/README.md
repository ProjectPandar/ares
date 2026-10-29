# project-settings-bool-wire

Fixture for the `project_settings.config` boolean wire-form RED
(`Config.cpp:DynamicPrintConfig::load_from_json` scalar else-branch,
`Config.hpp:1828-1848` `ConfigOptionBool::deserialize`,
`Config.hpp:96-99` `enum_looks_like_true_value`). See
`docs/architecture/ard-project-settings-bool-wire.md`.

`k1-staggered-json-true.3mf` is the stock Creality K1 bridge project exported
at Native runtime by the official 2.4.2 reference (`--export-3mf`, arrange
enabled) from the K1 stock presets (`seam_position: aligned`,
`seam_gap: 10%`, `staggered_inner_seams: "1"`), with `staggered_inner_seams`
rewired to an unquoted JSON `true` — the exact patch artifact of the seam
scan. Upstream `load_from_json` hits the `invalid json type` else-branch for
the boolean, drops the key, and slices UNSTAGGERED (first inner wall
`X130.857`); Ares main resolves the JSON boolean as `true` and staggers
(`X131.357`). The encoding is never produced by Orca's own writer
(`save_to_json` emits strings): this is a wire-form conformance witness.

Controls (same base project): stock `"1"` and probe `""`/`"0"` are
full-output PASS dual-stable — the divergence is isolated to the JSON
boolean. Bilateral value-matrix traces confirmed the wire-form classification
and complete-stream parity of these controls.

The fixture contains model/settings only — no G-code, no golden output.

`k1-staggered-json-true.3mf` SHA256:
`803a4bf91e7259068f7c1704c34454dde301281708a5d04b92fe741869a4eb86`
