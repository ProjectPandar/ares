# Current-run Native oracle cache handoff

This is a CI transport decision, not a slicing feature or rewrite milestone.
It does not change the upstream behavior implemented by any oracle recipe.

## Required boundaries

- Every oracle producer publishes its explicit output roots and every runtime
  dependency without a `cache.nixos.org-1` signature. Explicit roots are always
  published, including the signed GDB and patchelf suppliers.
- Signed Nixpkgs dependencies remain with `https://cache.nixos.org`. Consumers
  may not obtain in-run Native outputs from Hestia or rebuild missing outputs.
- Each Actions cache key contains the workflow run ID and producer variant.
  Consumers supply no `restore-keys`, require `fail-on-cache-miss`, and reject
  non-exact `cache-hit` results before attempting Native comparisons.
- The seed replay ELF, debug output and build source remain outputs/inputs of
  one derivation. The cache transport must not mix their suppliers.
- No generated Native G-code or golden is committed. Comparisons still execute
  Native at runtime; this cache only transports the compiled suppliers.

## Transport and trust

`native-oracle-cache.py` uses Nix's store export/import format, which preserves
store-path references. A shallow `nix copy` to a binary cache rejects missing
references; exporting the selected store objects allows their signed public
runtime dependencies to stay with the official cache instead of duplicating
those dependencies in every producer archive.

The producer records the selected objects' NAR hashes and compresses the export.
After exact-run restoration, the consumer first realises the signed dependencies
with only the official substituter and `max-jobs=0`, then imports the Native
objects and checks their NAR hashes against the producer manifest. An already
present path must match that manifest too. There is no Native fallback.

Hestia remains an optional producer-side reuse mechanism. Its manifest may
reference evicted packs, so a successful producer job or hook registration is
not evidence that a cache-only consumer can retrieve an output. Consumers use
neither that manifest nor Hestia's NAR service for in-run suppliers.

## Capacity gate

Before extending the handoff, account for every producer archive separately;
counting a deduplicated union understates Actions cache usage. Check the total
against the repository's 10 GB limit before pushing, and include the duplicate
push/pull-request workflow footprint. Label measured sizes and conservative
bounds separately. An unverified or excessive total does not permit a push.

## References

- [Nix store export](https://nix.dev/manual/nix/latest/command-ref/nix-store/export)
  and [import](https://nix.dev/manual/nix/latest/command-ref/nix-store/import):
  exports do not implicitly include references; imports require those references.
- [Actions cache restore](https://github.com/actions/cache/tree/v4/restore):
  explicit keys, cache-miss failure and exact-hit output.
- [Hestia write pipeline](https://github.com/Mic92/hestia/blob/v3/src/pipeline.rs):
  paths already in the manifest are skipped by the write pipeline.
