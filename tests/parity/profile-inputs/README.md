# Explicit parity profile input checks

The raw Ares input is independent of Native's packaged resources:

- `ARES_PARITY_PROFILE_RESOURCES_DIR`: verified unpruned source `resources/`
  at commit `345efec64c7b95a05ce5cf96df0dde66a158835c`, profiles tree
  `1a0b4442f231034b13984b227ac5e15e23b92541`.
- `ARES_ORCA_RESOURCES_DIR`: unchanged Native resource identity.

From an explicitly selected Native environment (`ARES_REQUIRE_ORCA=1`) with an
absolute external `ARES_PARITY_ARTIFACT_ROOT`, run:

```sh
python3 tests/parity/profile-inputs/check.py
```

This drives one existing public nextest case three times. Missing binding,
nonexistent path, and a same-HEAD real checkout with changed profile bytes must
panic at the required input boundary, not skip or fall back. A fresh private
fixture is copied by Git; the supplied source/index is not modified. Test logs
and `profile-input-boundaries.json` remain in the external artifact root.
No mocks, private-state access, copied hash algorithm or expected G-code.

Positive provisioning uses all four original cases:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(per_object_skirt_support_raft::)' --no-fail-fast --test-threads 1
```

Each writes `ares-profile-input.json` with verified source provenance alongside
its real whole-output comparison artifacts. The upstream-owned strict footer/
M73 residuals are still asserted and reported; reaching them is supply-boundary
success, **not** strict G-code parity success. No parser/OPC decoder is added.
Actual disk contents are hashed in a separate Git object database/index. On
Windows, pinned executable modes are restored because checkout filesystem bits
cannot express them; contents/path set remain strict, without CRLF conversion.

## Main CI impact at 9f6ee320d

| Job | Impact |
|---|---|
| `parity-seeded` | New pinned sparse checkout/binding and negative E2E step; runs the four public cases |
| `parity-official` | Same provisioning/checks; Native AppImage resource identity unchanged |
| `native` / `self-hosted` | Workspace explicitly excludes `binary(orca_parity)`; no runtime profile binding needed |
| `web` | Does not invoke these grouped CLI cases; unchanged |
| `native-oracle-build` matrix | Does not run the cases; unchanged |
| `format`, Pages | No affected test invocation; unchanged |

Both affected jobs provision the independent source via the same composite
`.github/actions/parity-profile-input`. It sparse-checks out only
`resources/profiles` (plus Git's cone-mode root files), verifies the pinned
revision/tree and sets the new variable before any affected case runs. Native
`ORCA_REF` stays exactly as main declared it; no Native cutover is smuggled into
this slice. Provisioning is not evidence that different Native versions compare
equally. When PR4 integrates, review for exactly one profile provisioning step
per consumer job and no duplicate source checkout/binding. No new cached Native
outputs/artifacts or persistent environment edits are part of these checks.
