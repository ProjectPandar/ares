# Beta raw-profile supply: public RED and failure contract

This slice repairs profile-input provisioning only. It does not change Orca
packaging, Ares preset resolution or strict whole-output comparisons. Native
G-code, caches, unpacked resources and golden files stay outside Git.

## Pinned upstream boundary

- OrcaSlicer `v2.5.0-beta`: commit
  `345efec64c7b95a05ce5cf96df0dde66a158835c`.
- Original `resources/profiles` Git tree:
  `1a0b4442f231034b13984b227ac5e15e23b92541`.
- Upstream `scripts/build_preset_cache.sh:2-25,149-198` and
  `.github/workflows/build_orca.yml:631-646` replace packaged vendor manifests
  and listed JSON presets with OPC caches. `PresetCacheFormat.hpp:115-172`
  defines OPC as header + cereal dictionary/body, not ZIP.
- `PresetBundle.cpp:654-685,725-748,7935-7980` owns Native cache-only loading.
  That behavior is unchanged. The Rust destination is the existing grouped
  public test's JSON-directory input adapter, a temporary compatibility shell
  around `PresetBundle::read_vendor` / `parse_vendor_json`, not a new Ares format.

## Executed public RED

Serial build, explicit beta environment and external artifact root:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(per_object_skirt_support_raft::)' --no-fail-fast --test-threads 1
```

Nextest run `545d452a-6192-4f05-897d-53016364f4bf`: **four executed, four failed**,
0.070 seconds after compilation. All four reached the same genuine error:

```text
<Native-AppImage>/resources/profiles/OrcaFilamentLibrary/filament:
No such file or directory (os error 2)
```

The cases, without renaming:

1. `per_object_skirt_control_matches_native`
2. `per_object_skirt_single_raft_control_matches_native`
3. `per_object_skirt_support_red_matches_native`
4. `per_object_skirt_raft_red_matches_native`

This execution used pre-implementation source at `63b82ec82`. The entire grouped
case module is byte-identical at the independent base `9f6ee320d`:
Git blob `13160a3ee392e640895c2183015a1b07ce0b562e`. The failure occurs at its
profile-directory read before any core slicing; no fresh main-base result is
claimed by that record. Implementation must be independent of unmerged PR4.

Complete original nextest output, native environment identities and reproducible
runtime artifacts are retained externally. This is not a
mock/private-helper test or a newly generated expected output.

## Boundary contract / failure model (before implementation)

- Require explicit `ARES_PARITY_PROFILE_RESOURCES_DIR` when a Native-enabled
  case attempts raw JSON profile input. Missing, nonexistent or invalid roots,
  wrong source revision or mismatching actual filesystem profile tree panic.
- Never fall back to `ARES_ORCA_RESOURCES_DIR`, floating source, older profiles
  or another representation; never add a new skip. Existing Native-unavailable
  policy is unchanged.
- Compute the actual supplied disk tree, including changed and extra files;
  nominal `HEAD:resources/profiles` alone is not content verification.
  Use Git's own hashing, not a copied Git tree algorithm, and a fresh external
  object database/index so the supplied checkout is not modified.
- Record Ares profile provenance separately: tag, full commit, verified tree,
  explicit binding and actual root. Keep Native AppDir, binary/OPC data and
  `ARES_ORCA_RESOURCES_DIR`/Native resource hash untouched.
- Existing export cache keys already hash selected machine/process/filament
  JSON (`runner/cache.rs:90-118`); do not rewrite Native producer identity.
- Negative E2Es use the same public test entry: omitted binding, nonexistent
  path and a real same-HEAD checkout whose profile bytes were changed. Require
  actual test panic at the intended boundary, not a compile/setup failure.

## Exit and residual boundaries

Exit is **supply-boundary GREEN**: all four pass raw input verification and reach
unchanged real whole-output comparisons; the three input-boundary negatives
fail closed. It is not strict Native G-code GREEN. Do not modify/waive/filter
assertions or skip comparisons to conceal residuals.

The first provisioning execution (`8c38884f-2065-4bd0-8010-a8ed0bdabfea`, before
moving the implementation off PR4's base) reached all four real comparisons:

| Case | Actual first strict Native/API difference |
|---|---|
| control | Config footer lacks `compatible_process_expression_group` (line6905) |
| single-raft-control | Same missing footer field (line4045) |
| support | Native emits an M73 timing update before the next move (line202) |
| raft | Native emits an M73 timing update before the next move (line252) |

All four independently recorded the pinned Ares input provenance; API/CLI raw
bytes matched each other. All four strict Native comparisons remained RED.
These residuals belong to their footer/M73 upstream slices, not this provisioning
slice. A rebased implementation must rerun the boundary checks and report its
actual residuals honestly rather than inheriting a strict GREEN claim.

No persistent local environment change is made in this slice; its one-line
append diff is separate. OPC decoding, include/inheritance algorithms, footer
fields and M73 timing fixes are deferred to independent source-cited slices.
