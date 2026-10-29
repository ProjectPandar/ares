# Release spiral layer ranges

Source: `PrintObjectSlice.cpp:194-222`, `PrintApply.cpp:342-395,958-1027`,
upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Approved scope: `docs/architecture/spiral-single-range-spec.md` and
`docs/architecture/spiral-layer-ranges-native-probe.md`.

The source fixture is `../spiral-hole-projected-split/spiral.3mf` (C0).
`active.3mf` (R1) and `ignored.3mf` (R2) preserve every original ZIP member
payload and append only `Metadata/layer_config_ranges.xml`. It addresses
object ordinal1, not model resource2. Both author layer_height0.2 and
outer_wall_speed17; bounds are `[0,1000]` and `[0,0.00005]`, respectively.

C0 and R2 have one global default range; R2's interval is ignored by EPSILON.
R1 has two global ranges even though the default tail has no volume. Release
NDEBUG removes the multi-range debug assertion: its ordinary ranged slice
uses Regular mode, unlike C0/R2's upper PositiveLargestContour mode.

The E2E witnesses run two fresh Native replays before public API and CLI,
checking complete Native streams, physical layer counts and active F1020
words. R2 also compares its entire Native stream to C0: admission only.
API and CLI each compare complete ordered output against Native using only
the established generator-line normalization, and must equal each other raw.
Native streams, repeat qualification, API/CLI output/errors and comparison
JSON are repeatable artifacts under ARES_PARITY_ARTIFACT_ROOT. Run the
`spiral_layer_ranges` tests with required Native in both official/seeded modes;
no output golden, tolerance or geometry/layer/speed normalization is added.

Input SHA256:
- C0: `7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27`
- R1: `00cf2cd954a57cc4be34bb5eeca90b41fbe92cf017e75fcf51f3f29d0d116133`
- R2: `32d081707888f472bd00f127a78e56514cb0351bf5c9ba480f08c9ac217ba568`
