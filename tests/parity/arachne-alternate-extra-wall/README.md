# Arachne alternate extra wall inputs

Input-only closed 3MF meshes/settings; no Native G-code is committed.
Approved SPEC: `c3d238a8a`, `docs/architecture/arachne-alternate-extra-wall-spec.md`.
Source owner: `PerimeterGenerator.cpp::process_arachne:2131–2175,2224–2244`.

Run `python3 tests/parity/arachne-alternate-extra-wall/generate.py` from the
repository root. The recipe copies the existing O0 project container, authors
one watertight model volume, and changes settings deterministically. Every
directed mesh edge occurs once with its reverse; signed volume is positive.
Fixed ZIP timestamps and sorted settings reproduce all input SHA256s in
`provenance.json`. Model origin is centered at (110,110), Z=0..1.2 mm,
0.2 mm layers. Each pair differs only in `alternate_extra_wall`.

| Pair | Independent behavior witness |
| --- | --- |
| prism | odd-layer extra wall, even-layer preservation, actual sparse infill |
| density-zero | enabled option inert at density 0 |
| density-half | upstream integer conversion makes 0.5 inert |
| density-one | 1 activates the increment |
| single-top | `only_one_wall_top` is inert with zero top shell layers |
| stepped-top | `only_one_wall_top` is inert on a stepped footprint with zero top shell layers |
| single-first | `only_one_wall_first_layer` is inert with zero bottom shell layers |
| zero-walls | odd layers may gain one despite configured zero walls |
| two-islands | each footprint island gains its own wall and infill boundary |
| spiral | effective spiral dispatch stays unchanged on/off |

Public test module: `ares-cli/tests/orca_parity/arachne_alternate_extra_wall.rs`.
Each mode saves input, launch identity, Native admission/footer, emitted wall
crossings/infill witnesses, API/CLI outputs/statuses and complete strict byte
comparison outside the repository. Optional Native omission is explicitly
reported; required Native execution never skips. A Native rejection is a
failure, not parity evidence. Native wall witnesses supplement full equality.
