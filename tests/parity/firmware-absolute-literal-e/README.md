# Firmware absolute literal-E INPUT pair

Upstream GCode/GCodeProcessor.cpp4873–4890 process_G10/G11 forwards
literal configured E words into ordinary process_G1 interpretation.
Rust slice: processor/motion/firmware.rs E construction only. See
`docs/architecture/2026-10-21-firmware-absolute-literal-e.md`.

Actual official-exported INPUT3MFs only, copied unchanged. Sole GUI key
Use firmware retraction/use_firmware_retraction0→1. Both absoluteE,
Marlin, identical valid before-layer macro without manually injected G92;
Native-generated G92 remains untouched. Ordinary closed8-sided cylinder,
six layers, spiralOFF, Classic/one wall/no infill/top/bottom/brim/skirt.

Manifest freezes whole archive/every uncompressed ZIP payload SHA256 and
actual export provenance. Same ZIP inventory, only changed payload
Metadata/project_settings.config, only changed key above; all other
settings/payloads identical. No post-export mutation, expected output
snapshots or Native content constants/assertions in tests/manifest.

Two direct native_stream_case::check tests reuse main's double Native
replay and native_api_cli validated API/bare CLI complete comparisons.
No copied replay/normalization/JSON/orchestration or private output seam.
CPU11/thread1/job1, ARES_REQUIRE_ORCA=1 and explicit official/seeded Native
reference environments and artifact roots. Target:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(firmware_absolute_literal_e_)'
```

Formal RED requires controlPASS/positiveactualFAIL in both modes and
stable Native repeats. This processor difference currently manifests
M73-only: accepted strict parity RED, not a physical motion claim.
No M73/time/E/header/movement/config/trailing-line masks or numeric
normalization; ONLY existing generated-by identity/date normalization.
No ignore/expected-failure decoration. Disabled environment is NOT proof.
Shared helper retains complete streams/launch/comparison errors.

Freeze tests/inputs/helpers for later GREEN. Precision/G91/line id/cache/
viewer and other interpreter/consumer changes deferred. RED after SPEC
requires review BEFORE production change; no FULL without window.
Discovery/GDB evidence remains separate in
`${COORD_ROOT}/evidence/w5-firmware-absolute-literal-e/`,
not substituted for formal public shared-checker proof.
