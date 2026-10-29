# ARD: source-owned painted MMU / beam call order

Status: proposed with SPEC RR; not production authority.
Base2ed045a7f; upstream8500fcdcc.

## Decision

Port the literal `PrintObjectSlice.cpp:1207–1243` order: conical handling,
MMU apply, adjacent fuzzy stage if supported, then existing InterlockingGenerator,
then compensation. `MultiMaterialSegmentation.cpp:2169–2207` beam suppresses
side cutting only. Do not bypass producer/caps/merge or reinterpret beam as a
new Ares pipeline feature.

Rust ownership remains private project_slice load_stages handoffs and existing
region_slices::{painted,interlocking}. Move the actual consumer once, including
unpainted objects; no shadow registry, duplicate early/late paths, public API,
geometry replacement or fallback. Remove only painted+beam's fail-closed check
once the real post-MMU consumer is implemented under GREEN authorization.
Fuzzy annotation support is not part of this decision.

## Non-negotiable evidence gates

Native-first paired GUI-reachable frozen inputs; public success-required RED
before production. Whole ordered Native/API/CLI and raw API==CLI comparisons
remain unchanged. No movement-count goldens, expected-error PASS, phase-only
acceptance, filter/tolerance or hidden normalization. Discovery error/output
observations are not public acceptance. Existing active unpainted beam behavior
is a regression obligation even though the new unpainted discovery pair is
single-material and beam-inactive.

All scope expansion or underlying generator residuals stop for review. Preserve
historical seals; no rewriting old painted carrier/RED work as new evidence.
CPU18, one job/test/parity worker; exact-tip final FULL requires separate window.
Portable core, files≤400LOC; WASM compile is not browser runtime qualification.

See [SPEC](painted-beam-call-order-spec.md) for upstream citations, destinations,
input hashes, discovery results, failure inventory and included/deferred scope.
