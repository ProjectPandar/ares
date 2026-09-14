# Un-ported Option Domains (tracking spec)

Status: TRACKED — candidates for future milestone slices. These are the
three option-coverage FAIL domains whose values ares rejects as
un-implemented project features (source anchors in
`tests/parity/option-coverage-summary.md`):

| Domain | Failing values | Upstream anchor | Scope |
|---|---|---|---|
| `raft_layers` (coInt, min tested) | `max` | `PrintConfig.cpp:5161` | raft generation + emission (raft layers before the object, raft interface, raft separation) |
| `sparse_infill_pattern` (coEnum) | `adaptivecubic` | `PrintConfig.cpp:3017` | adaptive layer-height-aware infill (`FillAdaptive.cpp`) |
| `wall_generator` (coEnum) | `arachne` | `PrintConfig.cpp:7155` | Arachne variable-width walls (the `arachne_default_prisms` forensics test also fails on this bucket) |

A fourth failing domain, `top_surface_pattern/octagramspiral`, is NOT
un-ported — it diverges on content and is blocked by the closed lattice
milestone (`2026-09-15-orca-lattice-parity-milestone.md` §Stage-4
verdict); do not reopen that investigation.

Each port needs its own milestone spec/plan naming the upstream
file/class/function boundary, included vs deferred behavior, and the
ares destination (per the rewrite gate in AGENTS.md).
