# Task 22M Package 5 MSRV Signed-Zero Amendment

## Authority And Finding

This amendment is read with the approved Task 22M specification at
`docs/superpowers/specs/2026-07-19-ksr-fdmtest-v4-task22m-elephant-foot-slice-ordering.md`
(SHA-256 `5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff`),
its plan (SHA-256
`b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`),
and the Package 5 coverage-repair specification/plan (SHA-256
`f7a79e4a80fad9d91609e8934d7260bea62016a9076a00c86926fa216b790c17` /
`a19bbfd753690bf3fdb8696f95b25d5ba0c74d82c91e7a2f8bc2bc1097ea65cf`).
All other approved amendments remain authoritative.

Independent revalidation found one P1 portability failure outside the two
repaired coverage gaps. The repository MSRV and Tier 1 CI use Rust 1.91.0.
Under that toolchain,
`task22m_elephant_foot_banded_smoothing_preserves_strict_equality_branches`
fails at accumulated indices 1 and 8: production returns positive zero while
the test expects negative zero. Rust 1.96.1 returns negative zero for the same
`f32::max` call, so the local default toolchain had hidden the failure.

The fixed source remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. Its owning boundary is
`src/libslic3r/ElephantFootCompensation.cpp:465-532`, specifically
`std::max(laplacian, compensation[i])` at lines 526-528. C++ `std::max(a, b)`
returns `a` when `a < b` is false, including equality. The Rust port must
preserve that operand order instead of delegating equal signed-zero selection
to `f32::max`.

## Authorized Delta

Modify exactly these existing Rust paths:

- `crates/ares-core/src/project_slice/elephant_foot/profile.rs`;
- `crates/ares-core/src/project_slice/tests/elephant_foot/profile.rs`.

In production, replace only `laplacian.max(current[index])` with the direct
source-equivalent selection:

```rust
if laplacian < current[index] {
    current[index]
} else {
    laplacian
}
```

Do not normalize zero, add a helper, change arithmetic grouping, change the
band walk, or use total ordering. This expression also preserves the fixed
source first-operand result when comparison is false for unordered values.

In the existing strict-equality test, change only accumulated expected indices
1 and 8 from `0x8000_0000` to `0x0000_0000`. The immediate-case expectations
and all other accumulated bits remain unchanged. No test is added, renamed, or
removed.

The prior exact 62-path frame becomes an exact 64-path frame: the same 62 paths
plus this specification and its companion plan. The two modified Rust files
already belong to the original Task 22M manifest. No other path is authorized.

## Acceptance

The existing Rust 1.91.0 focused test is the deterministic RED. After the
two-line semantic repair:

- the focused strict-equality test passes under isolated Rust 1.91.0 and
  1.96.1 target directories;
- Task 22M is exactly 81/81 under Rust 1.91.0 and the default toolchain;
- the synthetic M aggregate remains 10,351 bytes / SHA-256
  `c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d`;
- the KSR M checkpoint remains 3,008,346 bytes / SHA-256
  `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`;
- Task 22L, strict all-target/all-feature core clippy, core all-feature WASM,
  rustfmt, LOC, macro/unsafe, and diff gates pass.

Any other production or test change, changed fixed identity, or attempt to
weaken bit-exact assertions requires another approved amendment.
