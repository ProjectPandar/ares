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

Hestia is a **read-only** optional producer-side reuse mechanism in Tier1,
including its consumer jobs. No post-build uploads/drain may republish the same
outputs into that optional pool. This action-input change does not alter host,
quota or the existing GC workflow. Its manifest may reference evicted packs,
so producer success is not consumer delivery proof. Consumers still use neither
that manifest nor Hestia's NAR service for in-run Native suppliers.

## Capacity gate and receipts

Account for every producer archive separately; counting a deduplicated union
understates Actions cache usage. `native-oracle-variants.json` is the shared
retained matrix/audit inventory (16 after B15's processor stable-pair retirement).

`native-oracle-cache-budget.py` requires a complete exact-key inventory of both
automatic push/PR runs, successful supplier matrices and source/ref identity.
Actual archive bytes for the pair must remain **below 5,000,000,000**. Missing,
evicted, duplicate or unknown entries BLOCK; they are not zero.

It also reconciles every live repository key through the Actions service:

| Pool | Strict service-byte ceiling |
|---|---:|
| Native delivery, all refs/runs | 5,000,000,000 |
| Optional Hestia packs/manifests | 1,500,000,000 |
| Rust build/deploy/native, npm, Playwright acceleration | 1,500,000,000 |
| Whole repository working envelope | 8,000,000,000 |

The remaining 2,000,000,000 bytes below decimal10GB are reserved margin, not an
additional upload allocation. Unknown key families/identity/size, incomplete or
duplicate census rows, a stale usage/list reconciliation or an exhausted pool
BLOCK. Cache REST does not expose creator runs for optional Hestia/Rust/npm
entries: preserve UNKNOWN creator and exact ref/time context, count their bytes,
and never fabricate source provenance. Native source identity is mandatory.

After each actual Native cache publish, `native-oracle-cache-receipt.py` queries
the exact run/variant/ref key and records run/attempt/head, cache ID/version,
created time and actual archive bytes in the producer log and step summary.
Missing/ambiguous metadata fails the producer step. `RECORDED` is a receipt,
not a capacity pass or a guarantee the entry will survive eviction. The capacity
job emits the complete per-key/family census to log/summary without uploading
another artifact or cache. Inner XZ/NAR sizes are diagnostic, not authority.

## Retention and admission rules — no automatic deletion

- Protect the selected current automatic pair, its active consumers and approved
  reruns. The gate blocks Native keys outside this protected run set, even if
  total bytes happen to fit. That is a request for owner retention review,
  **not** proof an outside key is safe to delete.
- Retain all16 supplier variants and seed ELF/debug/source pairing until the
  approved contract changes. A higher manifest generation does not prove packs
  unreachable; optional acceleration is not automatically superseded.
- Unique run keys otherwise accumulate. A replacement pair must have room for
  its peak overlap before publication. Do not serialize whole workflows in a
  way that deadlocks the paired push/PR capacity wait.
- Removal of superseded exact IDs or deliberate optional-family retirement is
  an irreversible external operation and requires the **human user's explicit
  approval**. No script here deletes caches, invokes GC or changes quota.
- The historical143-key Hestia proposal is not deletion authorization. Refresh
  exact IDs/versions/ref/size, ownership/reachability and quiescent writers/
  consumers before any separately approved retirement. Do not delete current
  Native suppliers, main caches or other branches' acceleration implicitly.
- Repository-wide writers share this envelope. This post-publication gate is
  detection, **not a reservation, lock or atomic admission system**. Read-only
  Tier1 inputs prevent future duplicate writes through that path but do not
  reclaim existing entries or control other workflows. Explicit coordination
  of publication windows and approved retention remains required; do not claim
  a green future steady state until real service inventories and consumers pass.

The current real repository remains over budget with evicted suppliers. A
successful implementation check of its BLOCK behavior does not unblock PR4.

## Explicit saved-response analysis

For local regression checks without credentials, the capacity CLI accepts
`--inventory-response <path>`. The file is an explicitly selected primary input,
not a network-error fallback. Its `native-actions-metadata-responses-v1` envelope
contains repository, capture time and unmodified real public REST responses
with exact resource/parameter records. Missing, duplicate or mismatched records
BLOCK without a network request. The report/log/summary identifies this as
**saved evidence analysis, not current live CI proof**.

Without that explicit option, CI still requests the real service using its
existing token; authentication/network failures BLOCK. CI does not pass saved
responses. No token needs to be read, injected or printed for local analysis,
and no private API mock, monkeypatch or copied gate algorithm is used.

## References

- [Nix store export](https://nix.dev/manual/nix/latest/command-ref/nix-store/export)
  and [import](https://nix.dev/manual/nix/latest/command-ref/nix-store/import):
  exports do not implicitly include references; imports require those references.
- [Actions cache restore](https://github.com/actions/cache/tree/v4/restore):
  explicit keys, cache-miss failure and exact-hit output.
- [Hestia write pipeline](https://github.com/Mic92/hestia/blob/v3/src/pipeline.rs):
  paths already in the manifest are skipped by the write pipeline.
