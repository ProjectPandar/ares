# ARD-0068: Per-slice random context and reproducible fuzzy-skin reference

User-authorized execution interface: the slicer Context owns its random
instance, initialized from an explicit seed. Keep this out of PrintConfig,
printer presets and project serialization. Core must not obtain OS entropy,
perform filesystem I/O or use a global/thread-local ambient random generator.
Adapters may choose a seed and pass it into core.

Upstream rewrite boundary: `Feature/FuzzySkin/FuzzySkin.cpp:25-30`
(`random_value`, mt19937 and uniform_real_distribution), `UniformNoise`,
`fuzzy_polyline` (:287-330), and `fuzzy_extrusion_line`.
Rust destinations are the public project byte-input slicing interface,
`crates/ares-core/src/project_slice/perimeters/classic/fuzzy_skin.rs` and
`crates/ares-core/src/perimeters/fuzzy_skin_seeded.rs`. Replace the active project's independent
hash-based point randomness with the verified upstream random sequence;
do not create a separate Ares slicing pipeline.

Same input and fresh Context with the same seed must reproduce full ordered
output. Concurrent independent slice jobs must not share RNG state.
A fixed seed alone is insufficient if parallel workers consume shared draws
in a scheduler-dependent order: determine and record an explicit stable
consumption/stream rule before implementation. Do not add a contended global
mutex or serialize the entire slicer merely to stabilize randomness.

Independent design review identifies required decisions before coding:
choose the exact Context ownership/reuse/failure lifecycle and seed width;
specify stable path identities or deterministic consumption order; bind the
reference standard library's mt19937-to-double distribution, draw count and
arithmetic sequence with independent reference values. Pass random state
explicitly to actual consumers, not through GenerationMetadata or the Copy
ProjectSource carrier. Cover initial spacing, subsequent spacing and
UniformNoise amplitude draws; coherent noise still consumes random spacing.
The public Context must not imply unrelated random consumers already obey it.

Lifecycle: a Context is owned by one project-slice task and consumed
by its public entry point, including on failure; no reset/reuse/continuation
API is needed. Its private mt19937 instance is initialized with a u32 seed.
The instance is initialized once and consumed continuously through the actual
classic polygon materialization order. No path hashing, reseeding or ambient
thread-local state is added to Ares.
The existing Rust classic materialization is already sequential; do not add
new serialization to unrelated stages. Independent jobs own independent Contexts.
The seed-only reference is run with single-CPU affinity for this initial
continuous-stream comparison. Parallel original replay remains unproven;
do not claim its scheduler-dependent streams are equivalent to this task-local
stream. A proposed polygon-reseed prototype was removed after the single-CPU
experiment offered a narrower source-preserving comparison. No deterministic
path stream partition patch is applied to the reference without specific
approval. Verify actual traversal/consumption equivalence through output.
The first active consumer scope is classic project fuzzy
geometry; Arachne fuzzy and unrelated deterministic RNGs remain explicit
follow-up rewrite slices.

The older Point2/STL fuzzy prototype remains outside this slice. The Context
does not claim to control that temporary scaffold.

libstdc++ GCC 15.3.0 distribution evidence is retained in
`${TMP_ROOT}/ares-random-contract.NQYUkA/distribution-1000.log`: four independent seeds,
1000 outputs each, two u32 draws per double (first low, second high),
crossing multiple 624-word state updates. The corresponding Rust log matches
all 4000 double bit patterns.
An all-UINT_MAX controlled engine exercises rounding to 1 and confirms
correction to bits 0x3fefffffffffffff. Bind actual test-build toolchain identity
before claiming matching semantics across standard libraries.
The pinned Nixpkgs build uses GCC 15.3.0 as well, confirmed by evaluating
the exact pinned stdenv compiler version.

The isolated seed-only source worktree is `${TMP_ROOT}/ares-orca-replay-source.u3RYzz`
at upstream 8500fcdcc, with only the entropy-to-seed substitution so far.
Nix 2.4.2 derivation `xangh263swrdjfn0cw2zsi3r3pzwyrd8` built successfully;
binary build ID is aac19f42f818cfb2fced55d820577bcac0db405a.
Wrapper SHA256 is d6666c5fa1f168e455af5e02254d25abe0127592e6c82915c0f289dffa5b4c77;
the wrapped ELF SHA256 is bb3576da4bbd0043da3558972932f2e872b2eae4ec9d1cb805b012701f4191fe.
`scripts/orca-replay/seed-only.patch` retains the exact source substitution.
The checked-in build recipe reproduces the same derivation with explicit inputs:

```sh
task_source=$(mktemp -d ${TMP_ROOT}/ares-orca-seed-source.XXXXXX)
git -C OrcaSlicer worktree add --detach "$task_source" 8500fcdcc
git -C "$task_source" apply --unidiff-zero "$PWD/scripts/orca-replay/seed-only.patch"
ORCA_REPLAY_SOURCE="$task_source" \
ORCA_REPLAY_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
nix build --impure --no-link --print-out-paths --file scripts/orca-replay/build.nix
```

Invoke only through `scripts/orca-seeded-parity.sh`, with explicitly selected
`ORCA_REPLAY_BIN`, `ORCA_REPLAY_SEED` and `ORCA_REPLAY_CPU`; the official wrapper
remains unchanged. The test artifact root must be fresh for each seeded run.
Two fresh normal-affinity runs retained in
`${TMP_ROOT}/ares-orca-seed-only-repeat.sG9ggq` differ in M73 positions and feedrates,
not only timestamps. Two single-CPU-affinity runs in
`${TMP_ROOT}/ares-orca-seed-only-single.c9chK7` differ only in the generator timestamp
for this one classic cube fixture. This is narrow self-replay evidence, not
general thread-independent replay or Ares/reference parity.

The user separately authorizes an isolated reproducible OrcaSlicer test build
with fixed random seeds and initialization of confirmed uninitialized fields.
Keep its source patch, build identity and invocation distinguishable from
the untouched official 2.4.2 reference. Never silently substitute it for
official-binary evidence. Initialization repair is not fuzzy random behavior;
validate and record it independently before including it in that build.

Include actual seeded fuzzy geometry, not an unused Context scaffold.
Retain ripple and non-fuzzy regressions, exact requests and full-byte comparisons.
For the current single-CPU diagnostic seam require full reference self-replay
across fresh processes and consecutive jobs. General parallel replay additionally
requires different thread counts and remains uncompleted. Do not carry old
official reference cache into seeded runs: use a fresh dedicated artifact root
for each reference build and seed until cache identity covers both.
Defer changing unrelated already-deterministic RNG consumers until their
upstream rewrite contracts require it; seed ownership does not authorize
altering their algorithms. Record which random consumers the Context governs.
