# Spec: 3MF project options are archive-owned

## Observable contract

`ares slice` rejects `--options` when the input is a 3MF project. A 3MF slice obtains all effective options from the archive; an external JSON file is neither merged nor silently ignored. STL slicing continues to require `--options`.

## Upstream boundary

OrcaSlicer 2.4.2 loads project configuration from the 3MF package in `src/libslic3r/Format/bbs_3mf.cpp`; Ares preserves that ownership at the `ares-cli` input dispatch seam before archive parsing. No legacy explicit-options fallback is retained for project input.

## Acceptance

A `.3mf` invocation with `--options` fails with a boundary-specific error even if the archive bytes are invalid, while existing STL option behavior remains unchanged.
