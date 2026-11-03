# Thin-gap-j1 nearest-seam embedding layer gate witnesses

Frozen Snapmaker J1 IDEX stock thin-gap witnesses for the
SeamPlacer.cpp:1071-1077 `regions_with_perimeter` layer gate.

| Input | SHA256 |
| --- | --- |
| wall-arachne.3mf | 5f822b0761a325deb15795f82dedb9a7d03a8c40304ed389edaf4e4543b351c7 |
| wall-classic.3mf | 482eb2e6a19b43a56cc13553dc9c8795ca9d4ccf9510743d45d236b547cab01d |

Original source evidence:
`${EVIDENCE_ROOT}/w3-geometry-discovery3-78d642702/thin-gap-j1/inputs/`
(wall-arachne.3mf, not regenerated). wall-classic.3mf is a dual-mode sibling:
sole effective delta `wall_generator arachne->classic` in
`Metadata/project_settings.config`; every other member byte-copied; rebuilt
zip with fixed 1980 timestamps. Native `--slice 0` exit 0, T0=40/T1=39.

No archived G-code or static Native reference output is embedded.

Upstream counts layers for embedding via
`region->perimeters.entities.size() > 0` per region
(GCode/SeamPlacer.cpp:1071-1077), then populates embedded distance
(`:1095-1097`) which the comparator's hidden gate (`:770-775`) consumes.
`m_perimeters` is filled only by generator append points
(LayerRegion.cpp:82-84,119-122; PerimeterGenerator.cpp:1568-1569,229-232,
1613-1621,2128,2471-2476). The public cases use unchanged
`native_stream_case::check`: stable dynamic Native twice FIRST, validated
byte API and actual CLI, strict complete ordered generator-only comparison.
wall-arachne is the RED target (actual strict FAIL before the separately
approved production fix); wall-classic runs the same witness under the other
generator mode. No expected rejection/panic catch, mask/tolerance or
hardcoded expected seam index. See
`docs/architecture/nearest-seam-embedding-layer-gate.md` for the source
boundary, causal trace and host replay limitations. No production correction
here.
