# Plate custom G-code emission fixtures

Cases for `docs/architecture/plate-custom-gcode-emission.md`. Each one is
`tests/parity/orca_cli_ender3.3mf` (cube10, one filament, layers 0.2 to 10.0
in steps of 0.2) plus a `Metadata/custom_gcode_per_layer.xml` entry. Two
cases also edit `Metadata/project_settings.config`: `template` sets
`template_custom_gcode`, and `by-object` sets `print_sequence`. The files
are written by `node tools/chain-probe/plate-custom-gcode-emission-fixtures.mjs`,
and the script lists what each case covers.

`crates/ares-cli/tests/orca_parity/runner/plate_custom_gcode_emission.rs`
compares Ares with Native G-code made at runtime by the orca_parity runner
(official and seeded). It also builds the two-filament cases at runtime from
the vg1 X1C cube10 export. No Native output is stored.
