# Modifier sparse-loss inputs

Frozen authored W3 object-spacing preflight inputs, reused unchanged by W6
under approved SPEC cae06ee57. These are slicing inputs, not Native output.
The source recipe is `/tmp/worker3-object-spacing-inputs.py`; its full manifest
is `/tmp/worker3-object-spacing-native-1tgobnaj/inputs/manifest.json`.
Geometry/part/plate packaging originates from tracked authored
`tests/parity/validation/modifier-wall-loops-classic.3mf`; the recipe scales its
body to 20x20x10 and replaces the modifier's wall-count option with sparse
options. Machine/process/filament settings came from the prior runtime A45
Creality Ender-3 project, as recorded in that manifest. No generated G-code,
Native expected stream or historical oracle receipt is stored here.

- A-C0: no modifier. SHA256 `d6ed45dca46d2b9f7a395df577083f37fb3f747e89b69be70eb956b9868484dd`.
- A-C1: rightmost-quarter modifier, same AdaptiveCubic/30%/0.45mm settings.
  SHA256 `7fd8549eacaff99b71fe3b5eaea58d365a545d831174255cdc2bec1a69005619`.
- A-C2: nonintersecting modifier, 60%/0.60mm, leaving the body unaffected.
  SHA256 `e117163a8c090d4a5611e20912cbdadd8f100ee4c16b2338ce63e65109b7c720`.

All cases use multiline1. Tests retain these complete input bytes, run
OrcaRunner's runtime Native slice/cache, then call the validated public API and
real CLI. Full ordered-byte/line-count/layer comparisons normalize generated-by
only; API and CLI must also be raw-identical. Positive sparse XY/E counts are
additional observations and never replace full-output equality.
