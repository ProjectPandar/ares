# Spec: Preset identity activation

Source ownership, Rust destinations and deferred boundaries: ARD-0057.
Reuse existing typed project identity and output; only repair its native
test-adapter activation shell.

- Three keys must preserve requested values through original export rather
  than silently accepting the original preset's identity.
- Generated and alternate identity values must appear in final config output
  and pass complete ordered G-code equality using the first official result.
- Activation must preserve unrelated preset data and the normal option field.
- Test the agreed activation/export/full-output seams; no comparator exemption,
  source pinning or production fallback.
- Require RED/GREEN, full Nextest, fmt/Clippy, Tier-1 compiler checks and
  independent six-axis repair/re-review before committing code and docs.
