# Explicit parity-reference selection

## Boundary and problem

This is CLI test infrastructure for the existing source-cited slicing
rewrite, not a new slicer feature. Upstream boundary is OrcaSlicer.cpp's
export/slice CLI; destination is scripts/orca-parity.sh and behavioral
runner tests. No core slicing behavior changes.

The wrapper selected the first matching Nix-store directory. On this host
that resolved to 06nd93b4z9swqin8y009mbfsnlx1201d-orca-slicer-2.4.2, whose
rrhyj66ja14g954kqldfqb9ry71icyz5 derivation injects CoolingBuffer logging.
Whether inactive instrumentation changes output is unproven. It is still
not the reference selected by the earlier official-AppImage plan. The
wrapper's assertion that builds were byte-equivalent was unsupported.
Previous output comparisons remain evidence for those supplied streams,
not proof of equality with the official binary.

## Scope and plan

Require explicit ORCA_APPDIR before library discovery; remove implicit
store fallback. Preserve exact argument forwarding and library assembly.
Regress missing selection and an explicitly selected executable using the
actual shell wrapper. Restore the official Ubuntu2404 x86-64 v2.4.2
AppImage, check its published release SHA-256, extract outside the repo,
and rerun strict comparisons. Run workspace/KSR and fmt/clippy, then commit
code and docs together.

Excluded: inventing output normalization, claiming toolchain equivalence,
modifying upstream, or treating configuration selection alone as full
artifact provenance. The test harness still permits an explicit diagnostic
binary; users must not present such runs as official-reference acceptance.

## Reproduction

Release: https://github.com/OrcaSlicer/OrcaSlicer/releases/tag/v2.4.2
Asset: OrcaSlicer_Linux_AppImage_Ubuntu2404_V2.4.2.AppImage
Published and downloaded SHA-256:
`d12fb8c8eac1aecd2dfb6377acd48f994f8fa439ed5292fa532dd82880f029fd`.
Extract with `--appimage-extract`, then explicitly set:

```sh
export ORCA_APPDIR=/path/to/squashfs-root
ARES_PARITY_OPTIONS=1 ARES_OPTION_KEYS=sparse_infill_pattern \
  ARES_PARITY_ARTIFACT_ROOT=/path/to/parity-official \
  cargo nextest run -r -p ares-cli --test orca_parity option_coverage
```

This pins a binary artifact, not source text or Rust module structure.

## Evidence

The first test attempt incorrectly hid bash from Command's executable
lookup by replacing PATH; discard that failure as implementation evidence.
After fixing the test, it runs against the old wrapper and fails with
exit 0 instead of required 127. The corrected wrapper passes both tests.

The verified official asset extracts and executes successfully. Its full
sparse-pattern sweep records 21 PASS / 5 DIVERGENT. Crosszag, Lightning,
line, lockedzag and tpmsfk remain divergent. This does not prove that
instrumentation caused earlier failures or that passing patterns are
stable across all runs.
