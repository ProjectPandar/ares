# Native tools

## Historical initialized KSR producer

`orca-probes/ksr-initialized.nix` restores the initialized KSR producer from
OrcaSlicer revision `8500fcdccaa10b5099ac20d252af3a7c560046f1` and the
committed `orca-probes/ksr-initialized-source.patch`. It does not reference an
existing historical source store path. Prepare a clean source tree from Git
and apply the four-file patch, from the repository root:

```sh
ksr_source=$(mktemp -d)
git -C OrcaSlicer archive 8500fcdccaa10b5099ac20d252af3a7c560046f1 | tar -C "$ksr_source" -xf -
git -C "$ksr_source" apply "$PWD/tools/orca-probes/ksr-initialized-source.patch"
nix-hash --type sha256 --base32 "$ksr_source"
```

This only extracts repository source and computes its NAR hash; it does not
write to the system store or build Native. The verified NAR SHA-256 is
`1fbg964by6qdk22cx5c6anfr9jxkb6zgv0q0inhbpfgd87683phs`.
With the source name `orca-slicer-2.4.2-seed-replay-source`, it reproduces
`/nix/store/hxvv0g1qlw9lwimj2km3ghxphrl9p4qf-orca-slicer-2.4.2-seed-replay-source`.

The expression inherits the original Orca package patches, configure flags,
dependencies and wrapping from pinned nixpkgs
`/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source`.
After source preparation, recipe verification uses only instantiation:

```sh
ORCA_KSR_INITIALIZED_SOURCE="$ksr_source" nix-instantiate tools/orca-probes/ksr-initialized.nix
```

Verified derivation:
`/nix/store/4n1w0b6jgnfcwb6m9jfssndribyyg3v6-orca-slicer-seed-replay-2.4.2.drv`.
Its package output is
`/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2`.
No realization or compilation is part of this verification.

The historical source patch seeds FuzzySkin and initializes `PrintObject::m_id`
and two concentric-fill parameters. It leaves `GCode.cpp`'s placeholder RNG
unchanged; the current replay seed patch is therefore not interchangeable.
See [the authority decision](../docs/architecture/ard-ksr-initialized-runtime-authority.md)
for the runtime comparison contract.
