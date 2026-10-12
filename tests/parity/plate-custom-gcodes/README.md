# Plate custom G-codes fixtures

`Metadata/custom_gcode_per_layer.xml` cases for
`docs/architecture/project-plate-custom-gcodes.md`. Each one is
`tests/parity/orca_cli_ender3.3mf` plus that entry, written by
`node tools/chain-probe/plate-custom-gcodes-fixtures.mjs` (the script lists
what every case covers).

`crates/ares-cli/tests/orca_parity/plate_custom_gcodes.rs` compares Ares with
the Native `Model::plates_custom_gcodes` dump made at runtime by
`ARES_PLATE_CUSTOM_GCODES_BIN` (`tools/chain-probe/plate-custom-gcodes.nix`).
No Native output is stored.
