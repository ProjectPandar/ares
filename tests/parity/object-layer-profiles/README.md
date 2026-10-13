# Stored object layer profile inputs

Inputs are authored from the tracked `object-range-layer-height.3mf`.
Run `generate.py` from the repository root to reproduce each ZIP and its
provenance. All original ZIP members except `layer_config_ranges.xml` are
preserved; variants add only the authored `layer_heights_profile.txt`.
No Native G-code or historical output receipt is stored here.

The profile belongs to original ModelObject ordinal 1, not resource ID 2.
`bbs_3mf.cpp:2824-2883/2087-2090` imports it and
`PrintObject.cpp:3805-3833` selects it before `Slicing.cpp:807-866`.
The public validated slice must preserve the entire ordered Native stream.
The absent variant is an inactive control; ramp and step must change its
physical layer schedule. Runtime Native uses OrcaRunner slice_case caching.
