# Custom G-code overview fixture

`overview.3mf` is `tests/parity/orca_cli_ender3.3mf` plus one
`Metadata/custom_gcode_per_layer.xml` entry, written by
`node tools/chain-probe/custom-gcode-overview-fixtures.mjs --write-fixture`
(the script lists what every item covers). It is the input of
`docs/architecture/web-custom-gcode-overview.md`.

`crates/ares-vgcode/tests/custom_gcode_overview.rs` compares Ares with the
Native rows made at runtime by `ARES_CUSTOM_GCODE_OVERVIEW_BIN`
(`tools/chain-probe/custom-gcode-overview.nix`), with the item list from
`ARES_PLATE_CUSTOM_GCODES_BIN`. No Native output is stored.
