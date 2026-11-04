# ARD: 3MF + load-settings flows through the single composition with a format-keyed post-process gate

Decision for `docs/superpowers/specs/2026-10-10-cli-3mf-load-settings.md`.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Decisions (non-negotiable)

1. **No parallel load path.** Removing the CLI guard routes 3MF +
   `--load-settings`/`--load-filaments` through
   `from_3mf_with_profiles` — the same `:2750-2812` overlay the web
   profile path uses. A separate 3MF load-settings pipeline is not
   created.
2. **The `-19` gate is input-format-keyed.** `validate_print` in the
   composite entry is `true` exactly for 3MF inputs
   (`InputFormat::detect`), mirroring `OrcaSlicer.cpp:1682-1690` living
   inside the 3MF load block. It checks the **pre-overlay embedded**
   raw, not the composed view.
3. **The adapter's preset-file read stays first (known divergence,
   deferred).** The missing-preset-file ordering nuance (`-5` before
   `-19`, versus upstream's reverse) is deferred, not accepted: the
   follow-up CLI-adapter slice ports the `OrcaSlicer.cpp`
   file-read ordering (`:1682-1690` before `:1953-2024`).
4. **Two E2E REDs, both witness-derived.** The rejection witness pins
   the `-19` channels; the positive compatible case (embedded
   `post_process` emptied, loaded `compatible_printers` naming the
   machine) proves the forward slicing path by full-output byte
   comparison. If the positive RED exposes differences that GREEN
   cannot close, the slice is BLOCKED and the guard stays.

## Rejected alternatives

- Running the `-19` check in the CLI adapter: the gate belongs to the
  load seam that already owns it (`from_3mf_with_profiles` O5).
- Porting branches 1–2 of the `-17` adjudication in the same slice:
  separate upstream surface (`:2562-2606`), separate control; this
  slice stays bounded to the guard and the rejection witness.
