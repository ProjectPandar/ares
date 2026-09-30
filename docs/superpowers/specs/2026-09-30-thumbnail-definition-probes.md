# Spec: Thumbnail-definition parity probes

Source and Rust destination boundaries: ARD-0056. Existing typed normalization
is the rewrite under test; the CLI harness is only an external adapter.

- Generate canonical definitions for PNG/JPG/QOI/BTT_TFT/COLPIC, empty and
  multi-entry values; every requested canonical value must survive export.
- Preserve strict requested/effective proof. Shorthand conversion receives
  explicit independently specified expected values, not a generic exemption.
- Test format defaults and changed dimensions against original Orca, checking
  final config headers and complete ordered output with a single reference.
- Keep strings/GUI-rendered image coverage distinct from finite-format probes.
- Port the source-owned BTT header omission and skip header-only layer-count
  replacement. For those configured projects compare raw ordered bytes, with
  mutation rejection and accurate comparator/evidence metadata; preserve the
  ordinary generator contract unchanged.
- Require RED/GREEN, selected and full Nextest, fmt/Clippy, Tier-1 compiler
  checks, independent six-axis review, repair/re-review and code+docs commit.
