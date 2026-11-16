# CLI user preset source producer

## Rewrite boundary

Base: B59-26 `c0efd1742235db9d96faf0049d3b46645b538122`.
Upstream: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

This slice ports the ordinary selected-user JSON directory part of the lazy CLI
preset resolver, not an independently designed preset pipeline:

- `OrcaSlicer.cpp:2083–2109`: lazy AppConfig and read-only PresetBundle loading.
- `OrcaSlicer.cpp:2118–2157`: typed standalone bypass and user/system resolution.
- `OrcaSlicer.cpp:8237–8241`, `libslic3r/utils.cpp:360–370`: Native `--datadir`
  setup and attempted directory creation.
- `AppConfig.cpp:805–1040,1911–1926,1969–1972`, `AppConfig.hpp:161–162`:
  `OrcaSlicer.conf`, JSON `app.preset_folder`, missing-config defaults, and the
  CLI warning/reset path for invalid JSON.
- `PresetBundle.cpp:881–910,1411–1425,1505–1519`: system-first loading and exactly
  one selected user id (`default` when unset).
- `Preset.hpp:32–36`, `Preset.cpp:2032–2115`: the `machine`, `process`, and
  `filament` directories, their `base` directories, filename collection names,
  and resolve-before-commit directory loading.
- `Preset.cpp:1856–1959`: loaded user defaults, parent overlays, normalization,
  and discarding a preset with an invalid version or unresolved parent.
- `Semver.hpp:64–71`, `deps_src/semver/semver.c:142–219,553–569`: version
  acceptance, including Native's two-to-four numeric components.
- `PresetBundle.cpp:582–611`: external source admission by same-file identity.

## Non-negotiable boundaries

`ares-cli` owns filesystem paths, directory traversal, AppConfig decoding, and
same-open-handle file bytes/opaque identities. The only new CLI input is Native's
existing `--datadir`; there is no Ares-specific environment input or user-id flag.
`ares-core` never reads a directory, AppConfig file, environment variable, inode,
Windows file handle, or path fallback. WASM bindings are unchanged.

The adapter produces `LoadedPresetSources`. Its user collection transport holds
filename-derived collection names separately from authored document names and
opaque identities. Core resolves a directory against already loaded parents,
then commits that directory. The canonical byte-oriented loader reuses existing
source-cited preset defaults and normalization. The externally supplied source
list never self-registers into this transport. Admission remains kind + opaque
identity, not name, basename, bytes, or parent-name matching.

The B59-26 `add_loaded` API remains the host-canonical companion entry point. The
new collection entry point ports ordinary Native user-directory loading rather
than pretending raw inherited documents are already canonical loaded records.

## Empirical boundaries

The exact-argv Native E2E bypasses the general runner's automatic datadir
injection. Both official and seeded runs showed:

- No `--datadir`, with an isolated cwd `user/default` tree: no cwd user admission;
  Native exits 251 with `result.json.return_code=-5` and no G-code.
- Explicit isolated `--datadir`: user records are admitted and slicing succeeds.
- A valid AppConfig selects only `app.preset_folder`; an invalid JSON AppConfig
  resets to `default`; absent selected directories contribute no user records.
- A requested invalid-version/missing-parent record is discarded and its source
  subsequently fails admission. An unrelated malformed JSON file is skipped;
  it does not make the whole bundle fail.

Missing user directories are an empty contribution, not successful admission.
Typed files without inheritance do not require the producer. Explicit datadir
creation still happens during CLI setup, matching Native's setup phase.

## Included and deferred

Included: explicit Native datadir input, selected ordinary user JSON collections,
custom `base` roots, canonical user inheritance against already loaded records
and JSON system vendors, host identities, and narrow source-admission failures.

Deferred: `_local`/`_subscribed` bundle metadata and namespaces, `.info` sync
metadata, renamed-parent alias expansion, `.opc` vendor cache production, full
AppConfig migrations/selection UI state, browser-token producers, and WASM
producer bindings. A sparse external user filament without authored `filament_id`
also exposed a distinct CLI metadata mismatch (Native empty ID versus inherited
Ares ID); the inherited-option witness supplies its authored filament ID and does
not claim that adjacent metadata behavior. The JSON system-loader boundary is the existing B59-26 port;
this slice does not claim cache-only vendor installation parity.

The existing sparse typed-file compatibility shell is retained unchanged. A
separate exploratory typed `from:user`/no-inherits fixture exposed an adjacent
Ares `-17` compatibility discrepancy before producer loading; this slice does
not claim to fix that compatibility behavior. The setup/lazy positive control
uses an already supported standalone typed preset.

## Verification contract

Native/CLI E2Es preserve exact argv/cwd/exit/stdout/stderr and result artifacts in
isolated temp roots. Successful cases compare guard-normalized whole G-code.
Failures verify `-5` process/result channels and no G-code. Final qualification
also requires Linux fmt/clippy, Windows production check, WASM check, and
base-tip inventory comparisons in official and seeded modes with `P→F=0`.
