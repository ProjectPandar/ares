# Sequential template layer total — immutable Native-first witnesses

Source owner: pinned OrcaSlicer8500fcdcc `GCode.cpp::_do_export:2484–2528,2890`
and `process_layer:4677–4684`. See
[`sequential-template-layer-count-spec.md`](../../../docs/architecture/sequential-template-layer-count-spec.md).

Both new inputs are byte-for-byte copies of read-only discovery projects:

| Input | Discovery case | SHA256 | Native template total |
| --- | --- | --- | --- |
| `a1-byobject.3mf` | `a1-byobject-count-candidate/project.3mf` | `b436ba663498652206663fb1f276c7d972f09b5b6fccad15e1d998aeeb53e138` | 81 |
| `a1-bylayer.3mf` | `a1-stock-bylayer-control/project.3mf` | `4f0e4606b6725a207c07378093e3e169fa7610bb50cd8e24969ca50d3e1b5ace` | 50 |

The third test reuses unchanged `../auto-lift-query-frame/project.3mf` SHA256
`f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd`:
the original P1S ByLayer control, total50 and Native Auto-lift spiral intact.
No regeneration, synthetic template, geometry edit, or Native output golden.

Tests use unchanged `native_stream_case::check`, two fresh Native replays with
stock printer/sequence/offset, labels45/56, object names and all template layer
numbers checked before API/CLI. The independent Native count is 81 sequential,
50 ByLayer. Existing complete generator-only comparison and raw API/CLI
equality remain acceptance; local Native assertions do not replace comparison.
No separate preflight test or subsection-only public acceptance is used.

With canonical Native environment configured, select exactly these three tests:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(case_support::sequential_template_layer_count::)' \
  --no-fail-fast --test-threads 1
```

Qualification runs the canonical official and seeded entry points serially
with fresh per-mode caches, preserving commands, Native replay records, both
public G-code streams and full comparison JSON. RED expects the ByObject test
to fail the actual full comparator at byte55433/line1278 (`1/81` vs `1/50`),
while both ByLayer controls pass. The target is never expected-error inverted.
Later GREEN must pass these same tests unchanged.
