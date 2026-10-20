# Enum substitution parity

Upstream boundary: Model.cpp:259; Config.cpp:657-676;
Config.hpp:2032-2040; PrintConfig.cpp seam_position keys/default.
See docs/architecture/2026-10-19-project-settings-enum-substitution.md.

- `target-rearseam.3mf`: SHA256
  `86bd5b216652980a27bac87f0b234ee449b5b0dd4d8f8e16c987eb040a669dd5`.
  Approved unknown-key substitution witness from the original RED.
- `control-aligned.3mf`: SHA256
  `8056c75059017e981d9a15804eb66623d85e5f95a6a07981e151ab85135b1d9b`.
  Same ZIP entries; only project_settings.config payload changes, with
  exactly one semantic key difference, seam_position rear→aligned.

Formal RED-prime uses main's runner/native_api_cli.rs::NativeCase:
Native complete output → public slice_project_plate_validated → CLI slice.
Only the existing generator identity/date line is removed in acceptance.
Every case pins Native's emitted seam_position=aligned. NativeCase keeps
input, Native/API/CLI streams, launch logs and comparison.json under the
configured ARES_PARITY_ARTIFACT_ROOT, including both failing API/CLI
attempts. There is no borrowed PROFILE_ENTRY gate and no private seam.

Targeted nextest expression: `test(project_settings_enum_)`.
RED requires the target API/CLI load failure and passing aligned control;
GREEN requires both full-stream comparisons and raw API/CLI equality.

No FULL or integration evidence is claimed by the e81afae89 pre-window
preparation. Static gates: fmt, clippy workspace/all-targets -D warnings,
and browser wasm32-unknown-unknown check.
